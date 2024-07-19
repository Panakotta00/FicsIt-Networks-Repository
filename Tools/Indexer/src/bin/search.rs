use std::{env, io};
use std::fs::File;
use std::path::{Path, PathBuf};
use getopts::Options;
use tantivy::collector::TopDocs;
use tantivy::query::{FuzzyTermQuery, QueryParser};
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy};
use tempfile::TempDir;
use zip::ZipArchive;

fn unzip_index(index_file: &Path) -> zip::result::ZipResult<TempDir> {
	let index = File::open(index_file)?;

	let index_dir = TempDir::new().unwrap();

	let mut archive = ZipArchive::new(index)?;
	for i in 0..archive.len() {
		let mut file = archive.by_index(i)?;
		let file_name = file.name().to_owned();

		let target_path = index_dir.path().join(file_name);

		if let Some(parent_dir) = target_path.parent() {
			std::fs::create_dir_all(parent_dir)?;
		}

		let mut output_file = File::create(&target_path)?;

		io::copy(&mut file, &mut output_file)?;
	}

	Ok(index_dir)
}

fn load_index(index_dir: &Path) -> tantivy::Result<Index> {
	Index::open_in_dir(index_dir)
}

fn do_query(index: &Index, query: &str) -> tantivy::Result<()> {
	let schema = index.schema();

	let id = schema.get_field("id").unwrap();
	let name = schema.get_field("name").unwrap();
	let version = schema.get_field("version").unwrap();
	let short_description = schema.get_field("short_description").unwrap();
	let readme = schema.get_field("readme").unwrap();
	let eeprom_name = schema.get_field("eeprom_name").unwrap();
	let eeprom_title = schema.get_field("eeprom_title").unwrap();
	let eeprom_description = schema.get_field("eeprom_description").unwrap();

	let reader = index
		.reader_builder()
		.reload_policy(ReloadPolicy::OnCommitWithDelay)
		.try_into()?;

	let searcher = reader.searcher();

	let query_parser = QueryParser::for_index(&index, vec![id, name, version, short_description, readme, eeprom_name, eeprom_title, eeprom_description]);

	let query = query_parser.parse_query(query)?;

	let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

	for (_score, doc_address) in top_docs {
		let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
		println!("{}", retrieved_doc.to_json(&schema));
	}

	/*let query = query_parser.parse_query("title:sea^20 body:whale^70")?;

	let (_score, doc_address) = searcher
		.search(&query, &TopDocs::with_limit(1))?
		.into_iter()
		.next()
		.unwrap();

	let explanation = query.explain(&searcher, doc_address)?;

	println!("{}", explanation.to_pretty_json());*/

	Ok(())
}

fn cmd_search(index_file: &Path, query: &str) {
	let index_dir = unzip_index(index_file).unwrap();
	let index = load_index(index_dir.path()).unwrap();
	do_query(&index, query).unwrap();
}

fn main() {
	let args: Vec<String> = env::args().collect();
	let program = args[0].clone();

	let mut opts = Options::new();
	opts.optopt("i", "input", "set the search index file", "NAME")
		.optflag("h", "help", "print this help menu");
	let matches = match opts.parse(&args[1..]) {
		Ok(m) => m,
		Err(f) => panic!("{}", f.to_string()),
	};
	if matches.opt_present("h") {
		let brief = format!("Usage: {} [options] QUERY", program);
		print!("{}", opts.usage(&brief));
		return;
	}
	let input = matches.opt_str("i").unwrap_or("./index.zip".to_string());
	let query = if matches.free.is_empty() {
		"*".to_string()
	} else {
		matches.free.join(" ")
	};

	cmd_search(&Path::new(&input), &query);
}
