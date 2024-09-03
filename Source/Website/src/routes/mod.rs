pub mod middleware;
pub mod package;

use crate::repository::Repository;
use crate::routes::middleware::{AcceptJsonOnly, HTMXExtension};
use crate::templates::package::ListPackageResponse;
use crate::templates::{GetIndexResponse, GetPrivacyPolicyResponse, PackageCard};
use askama_axum::IntoResponse;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use axum::{http, Extension, Json};
use ficsit_networks_repository::index;
use ficsit_networks_repository::index::VersionData;
use futures_util::future::{join_all, try_join_all};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use tantivy::collector::{BytesFilterCollector, TopDocs};
use tantivy::query;
use tantivy::query::QueryClone;
use tantivy::schema::Value;

#[derive(Deserialize)]
pub struct SearchQuery {
	search: Option<String>,
	check_mods: Option<bool>,
	game_version: Option<String>,
	fin_version: Option<String>,
	#[serde(flatten)]
	other: HashMap<String, Option<String>>,
}

#[derive(Deserialize)]
pub struct Pagination {
	page: Option<usize>,
	page_size: Option<usize>,
}

#[derive(Clone)]
struct QueryVersions {
	fin_version: Option<semver::Version>,
	game_version: Option<semver::Version>,
	check_mods: bool,
	mods: HashMap<String, Option<semver::Version>>,
}

impl QueryVersions {
	fn parse_search_query(s: &SearchQuery) -> Self {
		let fin_version = s
			.fin_version
			.as_ref()
			.map(|s| semver::Version::parse(s).ok())
			.flatten();
		let game_version = s
			.game_version
			.as_ref()
			.map(|s| semver::Version::parse(s).ok())
			.flatten();
		let check_mods = s.check_mods.unwrap_or(false);
		let mods = s
			.other
			.iter()
			.map(|(k, v)| Some((k.strip_prefix("mod_")?, v)))
			.flatten()
			.map(|(k, v)| {
				Some((
					k.to_string(),
					v.as_ref().map(|v| semver::Version::parse(v).ok()).flatten(),
				))
			})
			.flatten()
			.collect();
		Self {
			fin_version,
			game_version,
			check_mods,
			mods,
		}
	}
}

fn check_version(requirement: Option<&String>, version: Option<&semver::Version>) -> Option<bool> {
	let requirement = semver::VersionReq::parse(requirement?).ok()?;
	Some(requirement.matches(version?))
}

fn check_versions(query: &QueryVersions, version_data: &VersionData) -> Option<()> {
	check_version(
		version_data.fin_version.as_ref(),
		query.fin_version.as_ref(),
	)
	.unwrap_or(true)
	.then_some(())?;
	check_version(
		version_data.game_version.as_ref(),
		query.game_version.as_ref(),
	)
	.unwrap_or(true)
	.then_some(())?;
	if query.check_mods {
		for m in &version_data.mod_dependencies {
			let version = query.mods.get(&m.id)?;
			if let Some(requirement) = &m.version {
				let version = version.as_ref()?;
				let requirement = semver::VersionReq::parse(&requirement).ok()?;
				if !requirement.matches(version) {
					return None;
				}
			}
		}
	}
	Some(())
}

pub async fn get_index(
	Extension(htmx): Extension<HTMXExtension>,
	Extension(AcceptJsonOnly(json_only)): Extension<AcceptJsonOnly>,
	State(repository): State<Repository>,
	Query(query): Query<SearchQuery>,
	Query(pagination): Query<Pagination>,
) -> axum::response::Result<Response> {
	let searcher = repository.reader.searcher();
	let schema = repository.package_schema.clone();

	let search = htmx
		.as_ref()
		.map(|uri| Query::try_from_uri(uri).ok())
		.flatten()
		.map(|q| {
			let mut q: SearchQuery = q.0;
			q.search = query.search.clone().or(q.search);
			q
		})
		.unwrap_or(query);
	let pagination = htmx
		.as_ref()
		.map(|uri| Query::try_from_uri(uri).ok())
		.flatten()
		.map(|q| {
			let mut q: Pagination = q.0;
			q.page = pagination.page.clone().or(q.page);
			q
		})
		.unwrap_or(pagination);
	let search_versions = QueryVersions::parse_search_query(&search);

	let query = search
		.search
		.as_deref()
		.filter(|s| !s.is_empty())
		.unwrap_or("*");

	let query_parser = tantivy::query::QueryParser::for_index(
		&repository.index,
		vec![
			schema.id,
			schema.name,
			schema.short_description,
			schema.readme,
			schema.tags,
			schema.versions,
			schema.authors,
		],
	);

	let query = query_parser
		.parse_query(&query)
		.ok()
		.unwrap_or(Box::new(query::AllQuery {}));

	let page_size = pagination.page_size.unwrap_or(10);
	let offset = pagination.page.unwrap_or(0) * page_size;

	let collector = TopDocs::with_limit(page_size).and_offset(offset);
	let collector_search_params = search_versions.clone();
	let collector = BytesFilterCollector::new(
		"version_data".to_string(),
		move |bytes: &[u8]| {
			if let Ok(version_data) = bitcode::decode::<index::VersionData>(bytes)
				.map_err(|e| println!("Error at decoding Version Data: {e}"))
			{
				check_versions(&collector_search_params, &version_data) == Some(())
			} else {
				false
			}
		},
		collector,
	);

	let top_docs = searcher.search(&query, &collector).ok().unwrap_or_default();

	let packages: Vec<PackageCard> = join_all(
		top_docs
			.into_iter()
			.map(|(_score, doc_address)| {
				let doc: tantivy::TantivyDocument = searcher.doc(doc_address).ok()?;

				let id = doc.get_first(schema.id)?.as_str()?.to_string();

				let version = if search_versions.fin_version.is_some() || search_versions.game_version.is_some() || search_versions.check_mods {
					let mut version: Vec<_> = doc.get_all(schema.version_data).zip(doc.get_all(schema.versions))
						.map(|(data, version)| Some((data.as_bytes()?, version.as_str()?)))
						.flatten()
						.map(|(data, version)| {
							Some((bitcode::decode::<VersionData>(data).ok()?, version))
						})
						.flatten()
						.map(|(data, version)| {
							check_versions(&search_versions, &data)?;
							Some(version)
						})
						.flatten()
						.map(|v| semver::Version::parse(v).ok())
						.flatten()
						.collect();
					version.sort();
					version.last().cloned()
				} else {
					None
				};

				let meta = repository.get_package_meta_by_id(id.clone());
				Some(async move { Some((id, version, meta.await.ok()?)) })
			})
			.flatten(),
	)
	.await
	.into_iter()
	.flatten()
	.map(|(id, version, meta)| PackageCard {
		id,
		name: meta.name,
		short_description: meta.short_description,
		version,
	})
	.collect();

	if json_only {
		Ok(Json(packages).into_response())
	} else {
		let next_page = pagination.page.unwrap_or(0) + 1;

		if htmx.is_some() {
			Ok(ListPackageResponse {
				packages,
				next_page,
			}.into_response())
		} else {
			Ok(GetIndexResponse {
				packages,
				next_page,
			}.into_response())
		}
	}
}

pub async fn privacy_policy() -> axum::response::Result<Response> {
	Ok(GetPrivacyPolicyResponse {}.into_response())
}
