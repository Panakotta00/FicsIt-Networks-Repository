use std::fs::File;
use std::path::Path;
use url::Url;

pub enum URLOrFile {
    URL(String),
    File(File),
}

pub async fn read_file_or_url(url: &str) -> Option<URLOrFile> {
    Some(if let Ok(url) = Url::parse(url) {
        let response = reqwest::get(url).await.unwrap();
        let content = response.text().await.unwrap();
        URLOrFile::URL(content)
    } else {
        let index = File::open(Path::new(url)).unwrap();
        URLOrFile::File(index)
    })
}