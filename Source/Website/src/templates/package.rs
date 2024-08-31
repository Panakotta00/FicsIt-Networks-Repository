use crate::templates::PackageCard;
use askama::Template;
use ficsit_networks_repository::metadata;
use ficsit_networks_repository::model::*;
use itertools::Itertools;

#[derive(Template)]
#[template(path = "package/list.html")]
pub struct ListPackageResponse {
	pub packages: Vec<PackageCard>,
	pub next_page: usize,
}

#[derive(Template)]
#[template(path = "package/package.html")]
pub struct GetPackageResponse {
	pub package: Package,
	pub version: Option<Version>,
}
