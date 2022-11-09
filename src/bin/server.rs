use actix_web::Error as ActError;
use actix_web::{
    error::ErrorBadRequest,
    middleware::Logger,
    web::{post, BytesMut, Payload},
    App, HttpResponse, HttpServer,
};
use anyhow::{Error as AnyError, Result};
use async_fs::File;
use clap::Parser;
use dotenv::dotenv;
use futures::{AsyncWriteExt, StreamExt};
use std::{env, fs};
use std::io::Write;
use std::process::{Command, Stdio, exit};
use local_ip_address::local_ip;

const MAX_SIZE: usize = 262_144; // 256k

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    // filename of pass_db
    #[clap(short, long, value_parser)]
    file_path: String,
}

async fn merge_request(payload: Payload) -> Result<HttpResponse, ActError> {
    let cli = Args::parse();

    // save the recieved db
    write_db(payload).await?;

    // merge the two db's
    merge(&cli.file_path).await?;

    // send the merged db back
    let body = async_fs::read(cli.file_path).await.unwrap();

    Ok(HttpResponse::Ok().body(body))
}

async fn write_db(mut payload: Payload) -> Result<(), ActError> {
    let mut body = BytesMut::new();

    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;

        // limit the max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk)
    }
    // body should now be the full database
    let filename = "new_db";
    let path = format!("./{}.kdbx", filename);
    let mut file = File::create(&path).await?;
    file.write_all(&body).await?;

    Ok(())
}

async fn merge(file_path: &str) -> Result<(), ActError> {
    let password = rpassword::prompt_password("Enter Password: ").unwrap();

    let mut cmd = Command::new("keepassxc-cli")
        .args(["merge", "-s", file_path, "new_db.kdbx"])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = cmd.stdin.take().expect("failed to take stdin");
    std::thread::spawn(move || {
        stdin.write_all(password.as_bytes()).unwrap();
    });

    let out = cmd.wait_with_output().unwrap();
    let out = std::str::from_utf8(&out.stdout).unwrap();
    println!("input: {}", out);

    fs::remove_file("./new_db.kdbx")?;

    Ok(())
}

#[actix_web::main]
async fn main() -> Result<(), AnyError> {
    Args::parse();
    dotenv().ok();
    let local_ip = local_ip().unwrap();
    println!("ip: {}", local_ip);
    let address = env::var("ADDRESS").unwrap();
    let port = env::var("PORT").unwrap();
    let url = format!("{}:{}", address, port);

    env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/", post().to(merge_request))
    })
    .workers(1)
    .bind(url)?
    .run()
    .await?;

    Ok(())
}
