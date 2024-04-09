use reqwest::blocking::Client;
use reqwest::Url;
use scraper::{Html, Selector};
use thiserror::Error;

use std::time::Duration;
use threadpool::ThreadPool;
use std::sync::mpsc;
use std::collections::HashSet;

#[derive(Error, Debug)]
enum Error {
    #[error("Request Error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Bad response HTTP: {0}")]
    BadResponse(String),
}

#[derive(Debug)]
struct CrawlCommand {
    url: Url,
    extract_links: bool,
}

fn visit_page(client: &Client, command: &CrawlCommand) -> Result<Vec<Url>, Error> {
    println!("check {:#}", command.url);

    let response = client.get(command.url.clone()).send()?;

    if !response.status().is_success() {
        return Err(Error::BadResponse(response.status().to_string()));
    }

    let mut link_urls = Vec::new();

    if !command.extract_links {
        return Ok(link_urls);
    }

    let base_url = response.url().to_owned();
    let body_text = response.text()?;
    let document = Html::parse_document(&body_text);

    let selector = Selector::parse("a").unwrap();
    let href_values = document
        .select(&selector)
        .filter_map(|element| element.value().attr("href"));
    for href in href_values {
        match base_url.join(href) {
            Ok(link_url) => {
                link_urls.push(link_url);
            }
            Err(err) => {
                println!("In {base_url:#} cannot parse {href:?}: {err}");
            }
        }
    }
    Ok(link_urls)
}


fn append_urls(r: Result<Vec<Url>, String>, url_checked: &mut HashSet<Url>, urls_to_check: &mut Vec<Url>) {
    match r {
        Ok(links) => {
            for link in links {
                if url_checked.insert(link.clone()) {
                    urls_to_check.push(link);
                }
            }
        },
        Err(e) => println!("{}", e),
    };
}

fn main() {
    let n_workers = 8;
    let pool = ThreadPool::new(n_workers);
    let (sender, receiver) = mpsc::channel();

    let start_url = Url::parse("https://www.google.org").unwrap();
    let mut urls_to_check = vec![start_url.clone();1];
    let mut url_checked: HashSet<Url> = HashSet::from_iter(vec![start_url;1]);

    let mut request_count = 0;
    const REQ_LIMIT: usize = 50;
    'outer: while request_count < REQ_LIMIT {
        while request_count < REQ_LIMIT && urls_to_check.len() > 0 {
            request_count += 1;
            let link = urls_to_check.pop().unwrap();
            let sender: mpsc::Sender<Result<Vec<Url>, String>> = sender.clone();
            pool.execute(move || {
                let client = Client::new();
                let crawl_command = CrawlCommand{ url: link, extract_links: true };
                match visit_page(&client, &crawl_command) {
                    Ok(links) => {
                        sender.send(Ok(links)).unwrap();
                    },
                    Err(e) => {
                        sender.send(Err(e.to_string())).unwrap();
                    }
                };
            });
        }
        std::thread::sleep(Duration::from_millis(10));
        while urls_to_check.len() == 0 {
            let r = receiver.try_recv();
            if pool.queued_count() == 0 && pool.active_count() == 0 && r.is_err() {
                break 'outer;
            }
            if r.is_ok() {
                append_urls(r.unwrap(), &mut url_checked, &mut urls_to_check);
            }
        }
    }
    pool.join();
    drop(sender);
    receiver.iter().for_each(|r| append_urls(r, &mut url_checked, &mut urls_to_check));

    println!("Result: ");
    url_checked.iter().for_each(|link| {println!("{}", link.to_string());});
    println!("Total urls: {:?}", url_checked.len());    
}