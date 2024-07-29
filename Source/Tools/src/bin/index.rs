use std::{env, fs};
use std::fs::File;
use std::path::Path;
use getopts::Options;
use tantivy::schema::*;
use tantivy::{Index, IndexWriter};
use tempfile::TempDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};
use ficsit_networks_repository::index::PackageSchema;
use ficsit_networks_repository::{index, metadata, model};
use ficsit_networks_repository::model::Readme;
use ficsit_networks_repository::model::Readme::Markdown;

fn do_index(input_dir: &Path, index_dir: &Path, schema: Schema, package_schema: &PackageSchema) -> tantivy::Result<Index> {
    let index = Index::create_in_dir(&index_dir, schema)?;

    let mut writer: IndexWriter = index.writer(50_000_000)?;

    for package_folder in fs::read_dir(input_dir)? {
        let package_folder = package_folder?;

        let package_id = package_folder.file_name().into_string().unwrap();

        let metadata_path = package_folder.path().join("metadata.toml");
        if !metadata_path.is_file() {
            println!("Package '{package_id}' has no metadata!");
            continue
        }
        let metadata: metadata::Package = match toml::from_str(fs::read_to_string(metadata_path)?.as_str()) {
            Ok(m) => m,
            Err(e) => {
                println!("Package '{package_id}' has invalid metadata: {e}");
                continue
            }
        };

        let mut versions: Vec<_> = fs::read_dir(package_folder.path())?
            .flatten()
            .map(|entry| {
                let file_name = entry.file_name();
                let version = file_name.to_str()?.strip_prefix("v")?;
                let version = semver::Version::parse(version).map_err(|e| println!("Package '{package_id}' has version folder '{version}' but cant be parsed as Semver: {e}")).ok()?;
                let metadata_path = entry.path().join("metadata.toml");
                if !metadata_path.is_file() {
                    println!("Package '{package_id}' Version '{version}' has no metadata!");
                    None?
                }
                let str = fs::read_to_string(metadata_path)
                    .map_err(|e| println!("Package '{package_id}' Version '{version}' has invalid metadata: {e}"))
                    .ok()?;
                Some((version.clone(), toml::from_str::<metadata::Version>(&str).map_err(|e| println!("Package '{package_id}' Version '{version}' has invalid metadata: {e}")).ok()?))
            })
            .flatten()
            .map(|(semver, v)| model::Version {
                version: semver,
                fin_version: v.fin_version,
                game_version: v.game_version,
                mod_dependencies: v.mod_dependencies.into_iter().map(|d| model::ModDependency{
                    id: d.id,
                    version: d.version,
                }).collect(),
                eeprom: v.eeprom.into_iter().map(|e| model::EEPROM{
                    name: e.name,
                    title: e.title,
                    description: e.description,
                }).collect(),
            })
            .collect();

        versions.sort_by(|v1, v2| v2.version.cmp(&v1.version));

        let readme_content = fs::read_to_string(package_folder.path().join("README.adoc"))
            .map(|s| Readme::ASCIIDOC(s))
            .or_else(|_| fs::read_to_string(package_folder.path().join("README.md")).map(|s| Readme::Markdown(s)))
            .unwrap_or(Readme::Markdown("".to_string()));

        let metadata = model::Package::from_metadata(package_id, readme_content, versions, metadata);

        index::add_package_to_index(&mut writer, package_schema, metadata).unwrap();
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

    let (schema, package_schema) = ficsit_networks_repository::index::build_schema();

    let index_dir = TempDir::new().unwrap();

    let index = do_index(input_dir, index_dir.path(), schema, &package_schema).unwrap();

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