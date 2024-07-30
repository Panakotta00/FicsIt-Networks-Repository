use crate::util::{read_file_or_url, read_file_or_url_as_string, URLOrFile};
use axum::http::StatusCode;
use ficsit_networks_repository::index::{load_schema, PackageSchema};
use ficsit_networks_repository::model;
use ficsit_networks_repository::model::Readme;
use ficsit_networks_repository::{index, metadata};
use futures_util::future::{join_all, try_join_all};
use moka::future::{Cache, CacheBuilder};
use std::borrow::Borrow;
use std::io::{Read, Seek};
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;
use tantivy::collector::TopDocs;
use tantivy::query::Occur;
use tantivy::schema::{IndexRecordOption, Value};
use tantivy::{query, Index, Score, Term};
use tempfile::TempDir;
use tokio::try_join;
use zip::ZipArchive;

#[derive(Clone)]
pub struct Repository {
	pub index: Arc<Index>,
	pub package_schema: Arc<PackageSchema>,
	pub reader: tantivy::IndexReader,
	pub raw_url: String,
	pub package_meta_cache: Arc<Cache<String, metadata::Package>>,
	pub package_cache: Arc<Cache<String, model::Package>>,
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
		let schema = index.schema();
		let package_schema = load_schema(&schema).unwrap();
		let index = Arc::new(index);
		let reader = index.reader_builder().try_into().unwrap();
		let package_meta_cache = CacheBuilder::new(10_000)
			.time_to_live(std::time::Duration::new(30, 0))
			.build();
		let package_cache = CacheBuilder::new(2_000)
			.time_to_live(std::time::Duration::new(30, 0))
			.build();

		Repository {
			index,
			package_schema: Arc::new(package_schema),
			reader,
			raw_url,
			package_meta_cache: Arc::new(package_meta_cache),
			package_cache: Arc::new(package_cache),
		}
	}

	pub fn path(&self, path: &str) -> String {
		self.raw_url.clone() + path
	}

	pub async fn get_package_index_data_by_id(&self, id: String) -> Option<Vec<semver::Version>> {
		let package_schema = self.package_schema.clone();
		let searcher = self.reader.searcher();
		tokio::task::spawn_blocking(move || {
			let query = query::BooleanQuery::new(vec![(
				Occur::Must,
				Box::new(query::TermQuery::new(
					Term::from_field_text(package_schema.id, &id),
					IndexRecordOption::Basic,
				)),
			)]);
			if let [(_, address)] = searcher.search(&query, &TopDocs::with_limit(1)).ok()?[..] {
				let doc: tantivy::TantivyDocument = searcher.doc(address).ok()?;
				Some(
					doc.get_all(package_schema.versions)
						.map(|v| semver::Version::parse(v.as_str()?).ok())
						.flatten()
						.collect(),
				)
			} else {
				None
			}
		})
		.await
		.ok()
		.flatten()
	}

	pub async fn get_version_meta(
		&self,
		id: &str,
		version: &str,
	) -> Result<metadata::Version, StatusCode> {
		let s = read_file_or_url_as_string(
			&self.path(&format!("/Packages/{id}/v{version}/metadata.toml")),
		)
		.await
		.ok_or(StatusCode::NOT_FOUND)?;
		toml::from_str(&s).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
	}

	pub async fn get_package_meta_by_id<S: Borrow<str>>(
		&self,
		id: S,
	) -> Result<metadata::Package, StatusCode> {
		let id = id.borrow();
		self.package_meta_cache
			.try_get_with_by_ref(id, async {
				println!("request!");
				let s = read_file_or_url_as_string(
					&self.path(&format!("/Packages/{id}/metadata.toml")),
				)
				.await
				.ok_or(StatusCode::NOT_FOUND)?;
				toml::from_str(&s).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
			})
			.await
			.map_err(|e| *e)
	}

	pub async fn get_package_readme_by_id(&self, id: &str) -> Option<model::Readme> {
		if let Some(readme) =
			read_file_or_url_as_string(&self.path(&format!("/Packages/{id}/README.md"))).await
		{
			Some(Readme::Markdown(readme))
		} else {
			read_file_or_url_as_string(&self.path(&format!("/Packages/{id}/README.adoc")))
				.await
				.map(|s| Readme::ASCIIDOC(s))
		}
	}

	pub async fn get_package_by_id(&self, id: &str) -> Result<model::Package, StatusCode> {
		self.package_cache
			.try_get_with_by_ref(id, async {
				let metadata = async { self.get_package_meta_by_id(id).await };
				let readme = async {
					self.get_package_readme_by_id(id)
						.await
						.ok_or(StatusCode::NOT_FOUND)
				};
				let versions = async {
					try_join_all(
						self.get_package_index_data_by_id(id.to_string())
							.await
							.ok_or(StatusCode::NOT_FOUND)?
							.into_iter()
							.map(|version| async {
								let metadata =
									self.get_version_meta(id, &version.to_string()).await?;
								Ok(model::Version {
									version,
									fin_version: metadata.fin_version,
									game_version: metadata.game_version,
									mod_dependencies: metadata
										.mod_dependencies
										.into_iter()
										.map(|m| model::ModDependency {
											id: m.id,
											version: m.version,
										})
										.collect(),
									eeprom: metadata
										.eeprom
										.into_iter()
										.map(|e| model::EEPROM {
											name: e.name,
											title: e.title,
											description: e.description,
										})
										.collect(),
								})
							}),
					)
					.await
				};

				let (metadata, readme, versions) = try_join!(metadata, readme, versions)?;

				Ok(model::Package::from_metadata(
					id.to_string(),
					readme,
					versions,
					metadata,
				))
			})
			.await
			.map_err(|e| *e)
	}
}
