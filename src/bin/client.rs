use anyhow::{Result, Error};
use async_fs::read;
use reqwest::Client;

async fn send_db(url: &str, filename: &str) -> Result<(), Error> {
    let filepath = format!("./{}.kdbx", filename);
    let bytes = read(&filepath).await?;

    let client = Client::new();
    let res = client.post(url)
        .body(bytes)
        .send()
        .await?;

    // wait for response from server
    println!("{:?}", res);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let url = "http://127.0.0.1:8080";
    let filename = "password";

    send_db(url, filename).await?;
    Ok(())
}
