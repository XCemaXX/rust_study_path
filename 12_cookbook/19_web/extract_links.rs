mod links {
    use std::error::Error;

    use select::{document::Document, predicate::Name};

    pub async fn print_links(page: &str) -> Result<(), Box<dyn Error>> {
        let res = reqwest::get(page).await?.text().await?;
        let doc = Document::from(res.as_str());
        let links = doc
            .find(Name("a"))
            .filter_map(|node| node.attr("href"))
            .into_iter()
            .filter(|l| l.starts_with("https://"))
            .take(5)
            .collect::<Vec<_>>();
        for link in links {
            println!("simple: {}", link);
        }

        Ok(())
    }
}

mod broken {
    use reqwest::StatusCode;
    use select::document::Document;
    use select::predicate::Name;
    use std::{collections::HashSet, error::Error};
    use url::{Position, Url};

    pub struct CategorizedUrls {
        pub ok: Vec<String>,
        pub broken: Vec<String>,
    }
    enum Link {
        GoodLink(Url),
        BadLink(Url),
    }

    async fn get_base_url(url: &Url, doc: &Document) -> Result<Url, url::ParseError> {
        let base_tag_href = doc.find(Name("base")).filter_map(|n| n.attr("href")).nth(0);
        let base_url =
            base_tag_href.map_or_else(|| Url::parse(&url[..Position::BeforePath]), Url::parse)?;
        Ok(base_url)
    }

    async fn check_link(url: &Url) -> Result<bool, reqwest::Error> {
        let res = reqwest::get(url.as_ref()).await?;
        Ok(res.status() != StatusCode::NOT_FOUND)
    }

    pub async fn check(page: &str) -> Result<CategorizedUrls, Box<dyn Error>> {
        let url = Url::parse(page)?;
        let res = reqwest::get(url.as_ref()).await?.text().await?;
        let doc = Document::from(res.as_str());
        let base_url = get_base_url(&url, &doc).await?;
        let base_parser = Url::options().base_url(Some(&base_url));
        let links: HashSet<_> = doc
            .find(Name("a"))
            .filter_map(|n| n.attr("href"))
            .filter_map(|l| base_parser.parse(l).ok())
            .collect();
        let mut tasks = vec![];
        let mut ok = vec![];
        let mut broken = vec![];

        links.into_iter().take(10).for_each(|link| {
            tasks.push(tokio::spawn(async move {
                if check_link(&link).await.unwrap_or(false) {
                    Link::GoodLink(link)
                } else {
                    Link::BadLink(link)
                }
            }));
        });
        for task in tasks {
            match task.await? {
                Link::GoodLink(url) => ok.push(url.to_string()),
                Link::BadLink(url) => broken.push(url.to_string()),
            }
        }
        Ok(CategorizedUrls { ok, broken })
    }
}

mod wiki {
    use std::{borrow::Cow, collections::HashSet, sync::LazyLock};

    use regex::Regex;

    pub fn print_unique_links(content: &str) {
        static WIKI_REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(
                r"(?x)
                \[\[(?P<internal>[^\[\]|]*)[^\[\]]*\]\]    # internal links
                |
                (url=|URL\||\[)(?P<external>http.*?)[ \|}] # external links
            ",
            )
            .unwrap()
        });

        let links: HashSet<Cow<str>> = WIKI_REGEX
            .captures_iter(content)
            .map(|c| match (c.name("internal"), c.name("external")) {
                (Some(val), None) => Cow::from(val.as_str()),
                (None, Some(val)) => Cow::from(val.as_str()),
                _ => unreachable!(),
            })
            .collect();
        for link in links.iter().take(5) {
            println!("wiki: {}", link);
        }
    }
}

#[tokio::main]
async fn main() {
    links::print_links("https://rust-lang-nursery.github.io/rust-cookbook/web/scraping.html")
        .await
        .unwrap();

    let categorized = broken::check("https://www.rust-lang.org/en-US/")
        .await
        .unwrap();
    for link in categorized.ok.iter().take(5) {
        println!("ok: {}", link);
    }
    for link in categorized.broken.iter().take(5) {
        println!("broken: {}", link);
    }

    let content = reqwest::get(
        "https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=raw",
    )
    .await
    .unwrap()
    .text()
    .await
    .unwrap();
    wiki::print_unique_links(&content);
}
