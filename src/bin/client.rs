use anyhow::{Error, Result};
use async_fs::{read, File};
use futures::AsyncWriteExt;
use reqwest::Client;
use clap::Parser;

async fn send_db(url: &str, file_path: &str) -> Result<(), Error> {
    let bytes = read(&file_path).await?;

    let client = Client::new();
    let res = client.post(url).body(bytes).send().await?;

    let bytes = res.bytes().await.unwrap();
    let filename = "Passwords.kdbx";
    let path = format!("./{}.kdbx", filename);
    let mut file = File::create(&path).await?;
    file.write_all(&bytes).await?;

    Ok(())
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    // filename of pass_db
    #[clap(short, long, value_parser)]
    file_path: String,

    // url of server
    #[clap(short, long, value_parser)]
    url: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let url = args.url;
    let file_path = args.file_path;

    send_db(&url, &file_path).await?;
    Ok(())
}
