use actix_web::Error as ActError;
use actix_web::{
    error::ErrorBadRequest,
    web::{post, BytesMut, Payload},
    App, HttpResponse, HttpServer,
};
use anyhow::{Error as AnyError, Result};
use async_fs::File;
use futures::{AsyncWriteExt, StreamExt};

const MAX_SIZE: usize = 262_144; // 256k

async fn merge_request(payload: Payload) -> Result<HttpResponse, ActError> {
    // save the recieved db
    write_db(payload).await?;

    // merge the two db's
    merge().await?;

    // send the merged db back
    Ok(HttpResponse::Ok().finish())
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

async fn merge() -> Result<(), ActError> {
    Ok(())
}

#[actix_web::main]
async fn main() -> Result<(), AnyError> {
    HttpServer::new(|| App::new().route("/", post().to(merge_request)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;

    Ok(())
}
