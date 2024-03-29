use std::fmt;
use std::fs;

use crate::http::HttpMethod;
use serde::de::Visitor;
use serde::{self, de::Error, Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub method: HttpMethod,
    pub pattern: String,
    #[serde(rename(deserialize = "queueDSN"))]
    pub queue_host: String,
    #[serde(rename(deserialize = "queueName"))]
    pub queue_name: String,
    #[serde(rename(deserialize = "buildEnv"))]
    pub build_env: String,
}

impl Config {
    pub fn parse_from_file(fname: &str) -> Result<Config, ConfigError> {
        let d = match fs::read_to_string(fname) {
            Ok(data) => data,
            Err(_) => return Err(ConfigError::new("could not open config file")),
        };

        let cfg: Config = match serde_yaml::from_str(&d) {
            Ok(cfg) => cfg,
            _ => return Err(ConfigError::new("could not deserialize config")),
        };

        Ok(cfg)
    }
}

impl<'de> Deserialize<'de> for HttpMethod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(HttpMethodVisitor)
    }
}

struct HttpMethodVisitor;

impl<'de> Visitor<'de> for HttpMethodVisitor {
    type Value = HttpMethod;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string of either \"GET\" or \"POST\"")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match v {
            "POST" => Ok(HttpMethod::POST),
            "GET" => Ok(HttpMethod::GET),
            _ => Ok(HttpMethod::GET),
        }
    }
}

#[derive(Debug)]
pub struct ConfigError {
    message: String,
}

impl ConfigError {
    pub fn new(msg: &str) -> ConfigError {
        ConfigError {
            message: msg.to_string(),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[cfg(test)]
mod config_tests {
    use crate::http::HttpMethod;

    use super::Config;
    use std::path::PathBuf;

    #[test]
    fn parse_happy() {
        let mut fname = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        fname.push("resources/test/happy-config.yaml");

        let r = Config::parse_from_file(&fname.to_string_lossy()).expect("error parsing conflict");

        assert_eq!(HttpMethod::GET, r.method);
        assert_eq!("amqp://some.host:port", r.queue_host);
        assert_eq!("QueueName", r.queue_name);
        assert_eq!("https://some.host/{PartOne}/{PartTwo}", r.pattern);
        assert_eq!("test", r.build_env)
    }
}
