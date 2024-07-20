use std::fs::File;
use std::path::Path;
use axum::body::Bytes;
use reqwest::StatusCode;
use tracing::error;
use url::Url;

pub enum URLOrFile {
    URL(Bytes),
    File(File),
}

pub async fn read_file_or_url(url: &str) -> Option<URLOrFile> {
    Some(if let Ok(url_str) = Url::parse(url) {
        let response = reqwest::get(url_str).await.map_err(|e| error!("Failed to request '{url}': {e}")).ok()?;
        if response.status() != StatusCode::OK {
            return None
        }
        let content = response.bytes().await.map_err(|e| error!("Failed to read bytes from response of '{url}': {e}")).ok()?;
        URLOrFile::URL(content)
    } else {
        let index = File::open(Path::new(url)).map_err(|e| error!("Failed to open file '{url}': {e}")).ok()?;
        URLOrFile::File(index)
    })
}