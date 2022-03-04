use serde_json::{Map, Value};
use std::error::Error;

#[derive(Debug, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum HttpMethod {
    GET,
    POST,
}

fn parse_message(msg: &str) -> Result<Map<String, Value>, Box<dyn Error>> {
    let parsed: Value = serde_json::from_str(msg)?;
    let map: Map<String, Value> = parsed.as_object().unwrap().clone();

    Ok(map)
}

#[cfg(test)]
mod http_tests {

    use super::parse_message;

    #[test]
    fn parse_happy() {
        let r = parse_message("{\"Key1\":\"One\", \"Key2\": \"Two\"}")
            .expect("could not parse message");

        assert_eq!("One", r.get("Key1").unwrap().as_str().unwrap());
    }
}
