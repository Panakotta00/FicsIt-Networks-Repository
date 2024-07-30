use crate::repository::Repository;
use crate::routes::middleware::HTMXExtension;
use crate::templates::package::{GetPackageResponse, ListPackageResponse};
use crate::util::{read_file_or_url, URLOrFile};
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response, Result};
use axum::{http, Extension};
use ficsit_networks_repository::index;
use ficsit_networks_repository::index::VersionData;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use tantivy::collector::{BytesFilterCollector, TopDocs};
use tantivy::schema::Value;

#[derive(Deserialize)]
pub struct PackageQuery {
	version: Option<String>,
}

pub async fn get_package(
	Extension(htmx): Extension<HTMXExtension>,
	State(repository): State<Repository>,
	Path((package_id)): Path<(String)>,
	Query(query): Query<PackageQuery>,
) -> Result<Response> {
	let package = repository.get_package_by_id(&package_id).await?;

	let version = query
		.version
		.map(|v| {
			let version = semver::Version::parse(&v).ok()?;
			package.versions.iter().find(|v| v.version == version)
		})
		.flatten()
		.or(package.versions.first());

	Ok(GetPackageResponse {
		package: &package,
		version,
	}
	.into_response())
}
