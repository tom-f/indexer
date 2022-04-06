use serde_json::{Map, Value};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum HttpMethod {
    GET,
    POST,
}

pub struct RequestBuilder {
    method: HttpMethod,
    pattern: String,
}

impl RequestBuilder {
    pub fn new(method: HttpMethod, pattern: String) -> RequestBuilder {
        RequestBuilder { method, pattern }
    }

    pub fn build(self, c: reqwest::Client, msg: &str) -> Option<reqwest::RequestBuilder> {
        match self.method {
            HttpMethod::GET => {
                let map = match map_message(msg) {
                    Some(m) => m,
                    None => return None,
                };

                Some(c.get(make_url(self.pattern, map)))
            }
            HttpMethod::POST => Some(c.post(self.pattern).body(String::from(msg))),
        }
    }
}

fn map_message(msg: &str) -> Option<HashMap<String, String>> {
    let value_map = match parse_message(msg) {
        Ok(vm) => match vm {
            Some(vm) => vm,
            None => return None,
        },
        Err(_) => return None,
    };

    if value_map.is_empty() {
        return None;
    }

    let m: HashMap<String, String> = value_map
        .iter()
        .map(|(k, v)| {
            let v = match v.clone() {
                Value::String(s) => s,
                _ => String::new(),
            };

            (k.clone(), v)
        })
        .collect();

    Some(m)
}

/// parse_message breaks down the provided message into a map or key -> value
fn parse_message(msg: &str) -> Result<Option<Map<String, Value>>, Box<dyn Error>> {
    let parsed: Value = serde_json::from_str(msg)?;
    let map: Option<Map<String, Value>> = parsed.as_object().map(|m| m.to_owned());

    Ok(map)
}

/// make_url takes a pattern and a map and returns a url.
/// The pattern is the base url, with placeholders form like: <Key>
/// The map is the key -> value map of a message.
/// The returned url is the base url with the placeholders replaced with the values.
fn make_url(pattern: String, map: HashMap<String, String>) -> String {
    let mut url = pattern;
    for (mut key, value) in map {
        if let Some(r) = key.get_mut(0..1) {
            r.make_ascii_uppercase();
        }

        let pattern = format!("<{}>", key);
        url = url.replace(&pattern, &value);
    }

    return url;
}

#[cfg(test)]
mod http_tests {

    use crate::http::RequestBuilder;

    use super::make_url;
    use super::map_message;
    use super::parse_message;
    use super::HttpMethod;

    use wiremock::matchers::method;
    use wiremock::matchers::path;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn parse_happy() {
        let r = parse_message(r#"{"Key1":"One", "Key2": "Two", "Key3":3}"#)
            .unwrap()
            .expect("could not parse message");

        assert_eq!("One", r.get("Key1").unwrap().as_str().unwrap());
        assert_eq!("Two", r.get("Key2").unwrap().as_str().unwrap());
        assert_eq!(3, r.get("Key3").unwrap().as_u64().unwrap());
    }

    #[test]
    fn parse_unhappy() {
        match parse_message(r#"{"Key1":"One", "Key2": "Two", Key3:3}"#) {
            Ok(_) => panic!("ok"),
            Err(_) => println!(
                "some error, which is what we expect. (we get an error back though which is nice)"
            ),
        }
    }

    #[test]
    fn map_message_happy() {
        match map_message(r#"{"Key1":"One", "Key2": "Two"}"#) {
            Some(map) => {
                assert_eq!(2, map.len());
                assert_eq!("One", map.get("Key1").unwrap());
                assert_eq!("Two", map.get("Key2").unwrap());
            }
            None => panic!("we got none"),
        }
    }

    #[test]
    fn map_message_unhappy() {
        match map_message("") {
            Some(_) => panic!("we got some"),
            None => println!("we got none"),
        }
    }

    #[test]
    fn make_url_happy() {
        let url = make_url(
            String::from("http://localhost:8080/api/v1/<Key1>/<Key2>"),
            vec![
                ("Key1".to_string(), "One".to_string()),
                ("Key2".to_string(), "Two".to_string()),
            ]
            .into_iter()
            .collect(),
        );

        assert_eq!("http://localhost:8080/api/v1/One/Two", url);
    }

    #[test]
    fn make_url_lowercase_keys() {
        let url = make_url(
            String::from("http://localhost:8080/api/v1/<Key1>/<Key2>"),
            vec![
                ("key1".to_string(), "One".to_string()),
                ("key2".to_string(), "Two".to_string()),
            ]
            .into_iter()
            .collect(),
        );

        assert_eq!("http://localhost:8080/api/v1/One/Two", url);
    }

    #[tokio::test]
    async fn request_builder_builds_correct_request() {
        let tests = vec![
            (
                HttpMethod::GET,
                "GET",
                "/api/v1/One/Two",
                "/api/v1/<Key1>/<Key2>",
                r#"{"Key1":"One", "Key2": "Two"}"#,
            ),
            (
                HttpMethod::POST,
                "POST",
                "/api/v1",
                "/api/v1",
                r#"{"Key1":"One", "Key2": "Two"}"#,
            ),
        ];

        let server = MockServer::start().await;

        for (http_method, method_name, endpoint, pattern, msg) in tests {
            let client = reqwest::Client::new();
            let request_builder = RequestBuilder::new(
                http_method,
                String::from(format!("{}{}", server.uri(), pattern)),
            );
            let request = request_builder.build(client, msg).unwrap();

            Mock::given(method(method_name))
                .and(path(endpoint))
                .respond_with(ResponseTemplate::new(200))
                .mount(&server)
                .await;

            let response = request.send().await.unwrap();

            assert_eq!(200, response.status().as_u16());
        }
    }
}
