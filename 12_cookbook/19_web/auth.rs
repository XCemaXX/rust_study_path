use reqwest::blocking::Client;

fn main() {
    let client = Client::new();

    let username = "testuser".to_owned();
    let password: Option<String> = None;

    let res = client
        .get("https://httpbin.org/")
        .basic_auth(username, password)
        .send();
    println!("{res:?}");
}
