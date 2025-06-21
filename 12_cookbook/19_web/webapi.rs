mod github {
    use std::{error::Error, time::Duration};

    use reqwest::{ClientBuilder, header::USER_AGENT};
    use serde::Deserialize;

    #[allow(dead_code)]
    #[derive(Deserialize, Debug)]
    struct GitHubContent {
        name: String,
        #[serde(rename = "type")]
        content_type: String,
        size: Option<u32>,
    }

    pub async fn query() -> Result<(), Box<dyn Error>> {
        let req = format!(
            "https://api.github.com/repos/{user}/{repo}/contents/",
            user = "XCemaXX",
            repo = "rust_study_path"
        );

        let client = reqwest::Client::new();
        let res = client
            .get(req)
            .header(USER_AGENT, "rust-web-api-client")
            .send()
            .await?;

        let users: Vec<GitHubContent> = res.json().await?;
        println!("{users:?}");

        Ok(())
    }

    pub async fn check_exist() -> reqwest::Result<()> {
        let user = "XCemaXX";
        let request_url = format!("https://api.github.com/users/{}", user);

        let timeout = Duration::new(5, 0);
        let client = ClientBuilder::new().timeout(timeout).build()?;
        let res = client
            .head(&request_url)
            .header(USER_AGENT, "GitHub-requires-it")
            .send()
            .await?;
        if res.status().is_success() {
            println!("{} is a user!", user);
        } else {
            println!("{} is not a user!", user);
        }
        Ok(())
    }
}

mod paginated {
    use std::collections::HashMap;

    use reqwest::header::USER_AGENT;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct ApiResponse {
        dependencies: Vec<DependencyData>,
        versions: Vec<VersionInfo>,
        meta: Meta,
    }

    #[derive(Deserialize)]
    struct DependencyData {
        version_id: u32,
    }

    #[derive(Deserialize)]
    struct VersionInfo {
        id: u32,
        #[serde(rename = "crate")]
        crate_name: String,
    }

    #[derive(Deserialize)]
    struct Meta {
        total: u32,
    }

    pub struct Dependency {
        pub crate_name: String,
    }

    pub struct ReverseDependencies {
        crate_id: String,
        dependencies: std::vec::IntoIter<Dependency>,
        client: reqwest::Client,
        page: u32,
        per_page: u32,
        total: u32,
        max_pages: u32,
    }

    impl ReverseDependencies {
        pub fn of(crate_id: &str) -> Self {
            Self {
                crate_id: crate_id.to_owned(),
                dependencies: vec![].into_iter(),
                client: reqwest::Client::new(),
                page: 0,
                per_page: 5,
                total: 0,
                max_pages: u32::MAX,
            }
        }

        pub fn with_max_pages(mut self, max: u32) -> Self {
            self.max_pages = max;
            self
        }

        pub async fn try_next(&mut self) -> Result<Option<Dependency>, reqwest::Error> {
            if let Some(dep) = self.dependencies.next() {
                return Ok(Some(dep));
            }
            if self.page >= self.max_pages
                || (self.page > 0 && self.page * self.per_page >= self.total)
            {
                return Ok(None);
            }
            self.page += 1;
            let url = format!(
                "https://crates.io/api/v1/crates/{}/reverse_dependencies?page={}&per_page={}",
                self.crate_id, self.page, self.per_page
            );
            println!("{}", url);

            let res: ApiResponse = self
                .client
                .get(&url)
                .header(USER_AGENT, "pager agent")
                .send()
                .await?
                .json()
                .await?;
            self.total = res.meta.total;

            let version_map: HashMap<u32, String> = res
                .versions
                .into_iter()
                .map(|v| (v.id, v.crate_name))
                .collect();
            let dependencies = res
                .dependencies
                .into_iter()
                .filter_map(|dep| {
                    version_map
                        .get(&dep.version_id)
                        .map(|crate_name| Dependency {
                            crate_name: crate_name.clone(),
                        })
                })
                .collect::<Vec<_>>();
            self.dependencies = dependencies.into_iter();

            Ok(self.dependencies.next())
        }
    }
}

#[tokio::main]
async fn main() {
    github::query().await.unwrap();
    github::check_exist().await.unwrap();

    let target_crate = "macroquad";
    let mut deps = paginated::ReverseDependencies::of(target_crate).with_max_pages(3);
    while let Some(dep) = deps.try_next().await.unwrap() {
        println!("{} depends on {}", dep.crate_name, target_crate);
    }
}
