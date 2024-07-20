pub mod model;

use std::io::{Read, Seek};
use std::path::Path;
use std::sync::Arc;
use tantivy::Index;
use tempfile::TempDir;
use url::Url;
use zip::ZipArchive;
use crate::util::{read_file_or_url, URLOrFile};

#[derive(Clone)]
pub struct Repository {
    pub index: Arc<Index>,
    pub reader: tantivy::IndexReader,
    pub raw_url: String,
}

fn unzip_index<R: Read + Seek>(reader: R) -> zip::result::ZipResult<TempDir> {
    let index_dir = TempDir::new().unwrap();

    let mut archive = ZipArchive::new(reader)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_name = file.name().to_owned();

        let target_path = index_dir.path().join(file_name);

        if let Some(parent_dir) = target_path.parent() {
            std::fs::create_dir_all(parent_dir)?;
        }

        let mut output_file = std::fs::File::create(&target_path)?;

        std::io::copy(&mut file, &mut output_file)?;
    }

    Ok(index_dir)
}

async fn get_and_unzip_index(url: &str) -> zip::result::ZipResult<TempDir> {
    match read_file_or_url(url).await.unwrap() {
        URLOrFile::URL(content) => unzip_index(std::io::Cursor::new(content)),
        URLOrFile::File(file) => unzip_index(&file),
    }
}

fn load_index(index_dir: &Path) -> tantivy::Result<Index> {
    Index::open_in_dir(index_dir)
}

impl Repository {
    pub async fn from_url(index_url: &str, raw_url: String) -> Repository {
        let index_file = get_and_unzip_index(index_url).await.unwrap();
        let index = load_index(index_file.path()).unwrap();
        let index = Arc::new(index);
        let reader = index.reader_builder().try_into().unwrap();

        Repository {
            index,
            reader,
            raw_url,
        }
    }

    pub fn path(&self, path: &str) -> String {
        self.raw_url.clone() + path
    }
}