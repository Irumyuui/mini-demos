use std::io::Write;

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

fn main() {
    let config = Config {
        url: "https://www.rust-lang.org".to_string(),
        accept: "text/javascript, application/javascript, application/ecmascript, application/x-ecmascript, */*; q=0.01".to_string(),
        accept_encoding: "gzip, deflate, br".to_string(),
        accept_language: "zh-CN,zh;q=0.9".to_string(),
        user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/77.0.3865.90 Safari/537.36', 'x-requested-with': 'XMLHttpRequest".to_string(),
    };

    let result = toml::to_string_pretty(&config).expect("Failed to serialize config");
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open("config.toml")
        .expect("Failed to open file")
        .write_all(result.as_bytes())
        .expect("Failed to write to file");
}
