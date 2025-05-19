use std::path::Path;

use reqwest::header::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
    url: String,
    accept: String,
    accept_encoding: String,
    accept_language: String,
    user_agent: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("{0}")]
    GenerateHeaders(#[from] reqwest::header::InvalidHeaderValue),
}

impl Config {
    pub async fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let context = tokio::fs::read_to_string(path).await?;
        let config = toml::from_str(&context)?;
        Ok(config)
    }

    pub fn get_headers(&self) -> Result<HeaderMap, ConfigError> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_str(&self.accept)?);
        headers.insert(
            ACCEPT_ENCODING,
            HeaderValue::from_str(&self.accept_encoding)?,
        );
        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_str(&self.accept_language)?,
        );
        headers.insert(USER_AGENT, HeaderValue::from_str(&self.user_agent)?);
        Ok(headers)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::read_from_file("config.toml").await?;
    let headers = config.get_headers()?;

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let response = client.get(&config.url).send().await?;
    println!("response:");
    println!("{:#?}", response);

    let text = response.text().await?;
    println!("text:");
    println!("{:#?}", text);

    Ok(())
}
