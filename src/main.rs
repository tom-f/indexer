mod config;
mod http;

use config::Config;
use http::RequestBuilder;

#[tokio::main]
async fn main() {
    let cfg = match Config::parse_from_file("") {
        Ok(cfg) => cfg,
        Err(message) => panic!("{:?}", message),
    };

    let msg = "message";

    let client = reqwest::Client::new();
    let rb = RequestBuilder::new(cfg.method, cfg.pattern);

    let req = match rb.build(client, msg) {
        Some(req) => req,
        None => panic!("no req"),
    };

    let res = match req.send().await {
        Ok(resp) => resp,
        Err(_) => panic!("over"),
    };

    println!("got the status {}", res.status())
}
