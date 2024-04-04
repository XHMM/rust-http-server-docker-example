use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    body::Bytes,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/health", get(health_check))
        .route("/compress", post(tiny_compress));

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    let addr = listener.local_addr().unwrap();

    println!("start listening on {}", addr);
    axum::serve(listener, router).await.unwrap();
}

async fn health_check() -> String {
    "OK".into()
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TinyApiRes {
    Ok(TinyApiOkRes),
    Error(TinyApiErrorRes),
}

#[derive(Debug, Deserialize)]
struct TinyApiOkRes {
    input: TinyApiOkInput,
    output: TinyApiOkOutput,
}

#[derive(Debug, Deserialize)]
struct TinyApiOkInput {
    size: u32,
    r#type: String,
}

#[derive(Debug, Deserialize)]
struct TinyApiOkOutput {
    height: u32,
    width: u32,
    ratio: f32,
    size: u32,
    r#type: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct TinyApiErrorRes {
    error: String,
    message: String,
}

struct AppError(anyhow::Error);

// Why not impl IntoResponse directly for anyhow::Error?
//
// Because only traits defined in the current crate can be implemented for types defined outside of the crate. So we need a wrapper around anyhow::Error
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

//
// Similar crates:
//   https://github.com/wyhaya/tinypng/tree/main
//   https://github.com/Danieroner/tinify-rs/tree/main
//
async fn tiny_compress(body: Bytes) -> Result<impl IntoResponse, AppError> {
    // Upload image to tinyapi
    let res = tiny_compress_upload(body).await?;

    // Download compressed image
    let url = res.output.url;
    let download_res = reqwest::get(url).await?;
    let bytes = download_res.bytes().await?;

    let image_type = res.output.r#type;
    let file_name = {
        let mime_type = image_type.parse::<mime::Mime>()?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        format!("{}.{}", now, mime_type.subtype())
    };

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, image_type.parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename={}", file_name)
            .parse()
            .unwrap(),
    );

    Ok((headers, bytes))
}

#[derive(thiserror::Error, Debug)]
enum TinyApiUploadError {
    #[error("api key missed")]
    NoApiKey(#[from] std::env::VarError),
    #[error("request error: {0}")]
    ReqErr(#[from] reqwest::Error),
    #[error("tinyapi returned an error: {}", .0.message)]
    ApiErr(TinyApiErrorRes),
}

/// Upload image to tinyapi
async fn tiny_compress_upload(body: Bytes) -> Result<TinyApiOkRes, TinyApiUploadError> {
    let client = reqwest::Client::new();
    let key = std::env::var("TINY_API_KEY")?;
    let res = client
        .post("https://api.tinify.com/shrink")
        .basic_auth("api", Some(key))
        .body(body)
        .send()
        .await?;

    let json = res.json::<TinyApiRes>().await?;
    match json {
        TinyApiRes::Ok(res) => Ok(res),
        TinyApiRes::Error(err) => Err(TinyApiUploadError::ApiErr(err)),
    }
}
