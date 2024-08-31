use std::fmt::{Display, Formatter, Write};
use serde::Serialize;
use crate::util::{serialize_semver_req, serialize_semver_req_opt, serialize_semver};
#[derive(Clone, Serialize)]
pub struct Package {
	pub id: String,
	pub name: String,
	pub short_description: String,
	pub readme: Readme,
	pub tags: Vec<String>,
	pub authors: Vec<String>,
	pub versions: Vec<Version>,
}

#[derive(Clone, Serialize)]
pub enum Readme {
	ASCIIDOC(String),
	Markdown(String),
}

impl Display for Readme {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Readme::ASCIIDOC(s) => f.write_str(s),
			Readme::Markdown(s) => f.write_str(s),
		}
	}
}

#[derive(Clone, Serialize)]
pub struct Version {
	#[serde(serialize_with="serialize_semver")]
	pub version: semver::Version,
	#[serde(serialize_with="serialize_semver_req_opt")]
	pub fin_version: Option<semver::VersionReq>,
	#[serde(serialize_with="serialize_semver_req_opt")]
	pub game_version: Option<semver::VersionReq>,
	pub mod_dependencies: Vec<ModDependency>,
	pub eeprom: Vec<EEPROM>,
}

#[derive(Clone, Serialize)]
pub struct ModDependency {
	pub id: String,
	#[serde(serialize_with="serialize_semver_req_opt")]
	pub version: Option<semver::VersionReq>,
}

#[derive(Clone, Serialize)]
pub struct EEPROM {
	pub name: String,
	pub title: String,
	pub description: String,
}

impl Package {
	pub fn from_metadata(
		id: String,
		readme: Readme,
		versions: Vec<Version>,
		metadata: crate::metadata::Package,
	) -> Self {
		Package {
			id,
			name: metadata.name,
			short_description: metadata.short_description,
			readme,
			tags: metadata.tags,
			authors: metadata.authors,
			versions,
		}
	}
}
