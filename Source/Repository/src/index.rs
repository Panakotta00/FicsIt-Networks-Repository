use crate::model::Package;
use bitcode::{Decode, Encode};
use tantivy::doc;
use tantivy::schema::*;
use tantivy::{Index, IndexWriter};

#[derive(Clone)]
pub struct PackageSchema {
	pub id: Field,
	pub name: Field,
	pub short_description: Field,
	pub readme: Field,
	pub tags: Field,
	pub authors: Field,
	pub versions: Field,
	pub version_data: Field,
}

#[derive(Encode, Decode)]
pub struct VersionData {
	pub fin_version: Option<String>,
	pub game_version: Option<String>,
	pub mod_dependencies: Vec<ModDependency>,
}

#[derive(Encode, Decode)]
pub struct ModDependency {
	pub id: String,
	pub version: Option<String>,
}

pub fn build_schema() -> (Schema, PackageSchema) {
	let mut builder = Schema::builder();

	let package = PackageSchema {
		id: builder.add_text_field("id", STRING | STORED | FAST),
		name: builder.add_text_field("name", TEXT),
		short_description: builder.add_text_field("short_description", TEXT),
		readme: builder.add_text_field("readme", TEXT),
		tags: builder.add_text_field("tags", STRING),
		authors: builder.add_text_field("authors", STRING),
		versions: builder.add_text_field("versions", STRING | STORED),
		version_data: builder.add_bytes_field("version_data", FAST | STORED),
	};

	(builder.build(), package)
}

pub fn load_schema(schema: &Schema) -> tantivy::Result<PackageSchema> {
	Ok(PackageSchema {
		id: schema.get_field("id")?,
		name: schema.get_field("name")?,
		short_description: schema.get_field("short_description")?,
		readme: schema.get_field("readme")?,
		tags: schema.get_field("tags")?,
		authors: schema.get_field("authors")?,
		versions: schema.get_field("versions")?,
		version_data: schema.get_field("version_data")?,
	})
}

pub fn add_package_to_index(
	index_writer: &IndexWriter<TantivyDocument>,
	package_schema: &PackageSchema,
	package: Package,
) -> tantivy::Result<tantivy::Opstamp> {
	let mut doc: TantivyDocument = doc!(
		package_schema.id => package.id,
		package_schema.name => package.name,
		package_schema.short_description => package.short_description,
		package_schema.readme => package.readme.to_string(),
	);

	for tag in &package.tags {
		doc.add_text(package_schema.tags, tag);
	}

	for author in &package.authors {
		doc.add_text(package_schema.authors, author);
	}

	for version in package.versions {
		let version_data = VersionData {
			fin_version: version.fin_version.map(|v| v.to_string()),
			game_version: version.game_version.map(|v| v.to_string()),
			mod_dependencies: version
				.mod_dependencies
				.into_iter()
				.map(|m| ModDependency {
					id: m.id.clone(),
					version: m.version.map(|v| v.to_string()),
				})
				.collect(),
		};

		doc.add_text(package_schema.versions, &version.version);
		doc.add_bytes(package_schema.version_data, bitcode::encode(&version_data));
	}

	index_writer.add_document(doc)
}
