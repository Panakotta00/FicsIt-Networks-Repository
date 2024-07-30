pub mod package;

use askama::Template;
use ficsit_networks_repository::metadata;
use itertools::Itertools;

#[derive(Clone)]
pub struct PackageCard {
	pub id: String,
	pub name: String,
	pub short_description: String,
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
