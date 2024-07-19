use std::{env, fs};
use std::fs::File;
use std::path::Path;
use getopts::Options;
use serde::Deserialize;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy};
use tempfile::TempDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

#[derive(Deserialize)]
struct PackageMetadata {
    name: String,
    version: String,
    short_description: String,
    #[serde(alias="EEPROM")]
    eeprom: Vec<EEPROMMetadata>,
}

#[derive(Deserialize)]
struct EEPROMMetadata {
    name: String,
    title: String,
    description: String,
}

fn build_schema() -> Schema {
    let mut builder = Schema::builder();

    builder.add_text_field("id", STRING | STORED);
    builder.add_text_field("name", TEXT);
    builder.add_text_field("version", STRING);
    builder.add_text_field("short_description", TEXT);
    builder.add_text_field("readme", TEXT);
    builder.add_text_field("eeprom_name", STRING);
    builder.add_text_field("eeprom_title", TEXT);
    builder.add_text_field("eeprom_description", TEXT);

    builder.build()
}

fn do_index(input_dir: &Path, index_dir: &Path, schema: Schema) -> tantivy::Result<Index> {
    let id = schema.get_field("id").unwrap();
    let name = schema.get_field("name").unwrap();
    let version = schema.get_field("version").unwrap();
    let short_description = schema.get_field("short_description").unwrap();
    let readme = schema.get_field("readme").unwrap();
    let eeprom_name = schema.get_field("eeprom_name").unwrap();
    let eeprom_title = schema.get_field("eeprom_title").unwrap();
    let eeprom_description = schema.get_field("eeprom_description").unwrap();

    let index = Index::create_in_dir(&index_dir, schema)?;

    let mut writer: IndexWriter = index.writer(50_000_000)?;

    for entry in fs::read_dir(input_dir)? {
        let entry = entry?;

        let package_id = entry.file_name().into_string().unwrap();

        let metadata_path = entry.path().join("metadata.toml");
        if !metadata_path.is_file() {
            println!("Package '{package_id}' has no metadata!");
            continue
        }
        let metadata: PackageMetadata = match toml::from_str(fs::read_to_string(metadata_path)?.as_str()) {
            Ok(m) => m,
            Err(e) => {
                println!("Package '{package_id}' has invalid metadata: {e}");
                continue
            }
        };

        let mut document = doc!(
            id => package_id,
            name => metadata.name,
            version => metadata.version,
            short_description => metadata.short_description,
        );

        for eeprom in metadata.eeprom {
            document.add_text(eeprom_name, eeprom.name);
            document.add_text(eeprom_title, eeprom.title);
            document.add_text(eeprom_description, eeprom.description);
        }

        if let Some(readme_content) = fs::read_to_string(entry.path().join("README.adoc"))
            .or_else(|_| fs::read_to_string(entry.path().join("README.md")))
            .ok() {
            document.add_text(readme, readme_content);
        }

        writer.add_document(document)?;
    }

    writer.commit()?;

    Ok(index)
}

fn zip_index(index_dir: &Path, index: &Index, output_file: &File) -> zip::result::ZipResult<()> {
    let mut zip = ZipWriter::new(output_file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::DEFLATE);
    for entry in &index.directory().list_managed_files() {
        zip.start_file(entry.file_name().unwrap().to_str().unwrap(), options)?;
        let mut file = File::open(index_dir.join(entry))?;
        std::io::copy(&mut file, &mut zip)?;
    }
    zip.finish().map(|_| ())
}

fn cmd_index(input_dir: &Path, output_file: &Path) {
    let out_file = File::create(output_file).unwrap();

    let schema = build_schema();

    let index_dir = TempDir::new().unwrap();

    let index = do_index(input_dir, index_dir.path(), schema).unwrap();

    zip_index(index_dir.path(), &index, &out_file).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("o", "output", "set the output file", "NAME")
        .optopt("i", "input", "set the input directory", "NAME")
        .optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!("{}", f.to_string()),
    };
    if matches.opt_present("h") {
        let brief = format!("Usage: {} FILE [options]", program);
        print!("{}", opts.usage(&brief));
        return;
    }
    let output = matches.opt_str("o").unwrap_or("./index.zip".to_string());
    let input = matches.opt_str("i").unwrap_or("./Packages".to_string());

    cmd_index(Path::new(&input), Path::new(&output));
}