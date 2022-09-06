use anyhow::{Result, Error};
use async_fs::{read, File};
use reqwest::Client;
use futures::AsyncWriteExt;
use std::env;
use dotenv::dotenv;

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

    let bytes = res.bytes().await.unwrap();
    let filename = "new_db";
    let path = format!("./{}.kdbx", filename);
    let mut file = File::create(&path).await?;
    file.write_all(&bytes).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    let address = env::var("ADDRESS").unwrap();
    let port = env::var("PORT").unwrap();
    let url = format!("http://{}:{}", address, port);
    let filename = env::var("DB_NAME").unwrap();

    send_db(&url, &filename).await?;
    Ok(())
}
