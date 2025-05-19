#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://httpbin.org/ip";
    let result = reqwest::get(url).await?.text().await?;
    println!("{:#?}", result);
    Ok(())
}
