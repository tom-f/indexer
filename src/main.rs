mod config;
mod http;

use reqwest::RequestBuilder;
// use crate::config::Config;

#[tokio::main]
async fn main() {
    let method_s = String::from("GET");

    let client = reqwest::Client::new();

    let method = match method_s.as_str() {
        "POST" => http::HttpMethod::POST,
        "GET" => http::HttpMethod::GET,
        _ => panic!("could not do it"),
    };

    let req = match method {
        http::HttpMethod::POST => make_post(client),
        http::HttpMethod::GET => make_get(client),
    };

    let res = match req.send().await {
        Ok(resp) => resp,
        Err(_) => panic!("over"),
    };

    println!("got the status {}", res.status())
}

fn make_post(c: reqwest::Client) -> RequestBuilder {
    c.post("")
}

fn make_get(c: reqwest::Client) -> RequestBuilder {
    c.get("https://google.com")
}
