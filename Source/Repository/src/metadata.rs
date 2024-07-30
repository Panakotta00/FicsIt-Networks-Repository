use crate::util::{deserialize_semver_req_opt, serialize_semver_req_opt};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Serialize, Deserialize)]
pub struct Package {
	pub name: String,
	pub short_description: String,
	pub tags: Vec<String>,
	pub authors: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Version {
	#[serde(
		serialize_with = "serialize_semver_req_opt",
		deserialize_with = "deserialize_semver_req_opt",
		default
	)]
	pub fin_version: Option<semver::VersionReq>,
	#[serde(
		serialize_with = "serialize_semver_req_opt",
		deserialize_with = "deserialize_semver_req_opt",
		default
	)]
	pub game_version: Option<semver::VersionReq>,
	#[serde(default)]
	pub mod_dependencies: Vec<ModDependency>,
	#[serde(alias = "EEPROM", default)]
	pub eeprom: Vec<EEPROM>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ModDependency {
	pub id: String,
	#[serde(
		serialize_with = "serialize_semver_req_opt",
		deserialize_with = "deserialize_semver_req_opt",
		default
	)]
	pub version: Option<semver::VersionReq>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EEPROM {
	pub name: String,
	pub title: String,
	pub description: String,
}
