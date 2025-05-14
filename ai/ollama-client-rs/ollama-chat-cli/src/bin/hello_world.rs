#![allow(unused)]

use std::io::Write;

use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};
use tokio::io::{self, AsyncWriteExt};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ollama = Ollama::default();

    let model = "deepseek-r1:14b";
    let prompt = "Hello world!";

    let mut stream = ollama
        .generate_stream(GenerationRequest::new(model.into(), prompt))
        .await?;

    let mut stdout = io::stdout();
    while let Some(res) = stream.next().await {
        let responses = res?;
        for resp in responses {
            stdout.write_all(resp.response.as_bytes()).await?;
            stdout.flush().await?;
        }
    }

    Ok(())
}
