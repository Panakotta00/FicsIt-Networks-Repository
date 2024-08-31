use crate::repository::Repository;
use crate::routes::middleware::{AcceptJsonOnly, HTMXExtension};
use crate::templates::package::{GetPackageResponse, ListPackageResponse};
use crate::util::{read_file_or_url, URLOrFile};
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response, Result};
use axum::{http, Extension, Json};
use ficsit_networks_repository::index;
use ficsit_networks_repository::index::VersionData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use tantivy::collector::{BytesFilterCollector, TopDocs};
use tantivy::schema::Value;
use ficsit_networks_repository::model::{Package, Version};

#[derive(Deserialize)]
pub struct PackageQuery {
	version: Option<String>,
}

#[derive(Serialize)]
pub struct PackageJsonResponse {
	pub package: Package,
	pub version: Option<Version>,
}

pub async fn get_package(
	Extension(AcceptJsonOnly(json_only)): Extension<AcceptJsonOnly>,
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
		.or(package.versions.first())
		.cloned();

	if json_only {
		Ok(Json(PackageJsonResponse{
			package,
			version
		}).into_response())
	} else {
		Ok(GetPackageResponse {
			package,
			version,
		}
			.into_response())
	}
}
