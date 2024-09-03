pub mod package;

use askama::Template;
use ficsit_networks_repository::util;
use itertools::Itertools;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct PackageCard {
	pub id: String,
	pub name: String,
	pub short_description: String,
	#[serde(serialize_with="util::serialize_semver_opt")]
	pub version: Option<semver::Version>,
}

#[derive(Template)]
#[template(source = "", ext = "")]
pub struct EmptyResponse {}

#[derive(Template)]
#[template(path = "index.html")]
pub struct GetIndexResponse {
	pub packages: Vec<PackageCard>,
	pub next_page: usize,
}

#[derive(Template)]
#[template(path = "privacy-policy.html")]
pub struct GetPrivacyPolicyResponse {}
