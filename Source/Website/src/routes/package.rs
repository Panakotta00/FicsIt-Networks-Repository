use axum::Extension;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Result, IntoResponse, Response};
use serde::Deserialize;
use tantivy::collector::TopDocs;
use tantivy::schema::Value;
use crate::repository::Repository;
use crate::templates::package::{GetPackageCardResponse, GetPackageResponse, ListPackageResponse};
use crate::util::{read_file_or_url, URLOrFile};

#[derive(Deserialize)]
pub struct SearchQuery {
    search: Option<String>,
}

#[derive(Deserialize)]
pub struct Pagination {
    page: Option<usize>,
    page_size: Option<usize>,
}

pub async fn get_package_list(
    Extension(htmx): Extension<bool>,
    State(repository): State<Repository>,
    query: Query<SearchQuery>,
    pagination: Query<Pagination>,
    headers: HeaderMap,
) -> Result<ListPackageResponse> {
    let package_ids = || -> Option<_> {
        let searcher = repository.reader.searcher();
        let schema = repository.package_schema;

        let query_parser = tantivy::query::QueryParser::for_index(&repository.index, vec![
            schema.id, schema.name, schema.short_description, schema.readme, schema.tags, schema.versions, schema.authors]);

        let url = headers.get("HX-Current-URL").map(|s| s.to_str().ok()).flatten().map(|s| url::Url::parse(s).ok()).flatten();

        let mut query = query.search.clone().or_else(|| {
            Some(url.as_ref()?.query_pairs().find(|(k, v)| k == "search")?.1.to_string())
        }).unwrap_or("*".to_string());
        if query.is_empty() { query = "*".to_string(); }
        let query = query_parser.parse_query(&query).ok()?;

        let page_size = pagination.page_size.or_else(|| {
            Some(url.as_ref()?.query_pairs().find(|(k, v)| k == "page_size")?.1.parse().ok()?)
        }).unwrap_or(10);
        let offset = pagination.page.or_else(|| {
            Some(url?.query_pairs().find(|(k, v)| k == "page")?.1.parse().ok()?)
        }).unwrap_or(0) * page_size;

        let collector = TopDocs::with_limit(page_size).and_offset(offset);
        let top_docs = searcher.search(&query, &collector).ok()?;

        Some(top_docs.into_iter().map(|(_score, doc_address)| {
            let doc: tantivy::TantivyDocument = searcher.doc(doc_address).ok()?;
            doc.get_first(schema.id).map(|v| v.as_str()).flatten().map(|v| v.to_string())
        }).flatten().collect())
    }().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(ListPackageResponse {
        package_ids,
    })
}

#[derive(Deserialize)]
pub struct PackageQuery {
    version: Option<String>,
}

pub async fn get_package(
    Extension(htmx): Extension<bool>,
    State(repository): State<Repository>,
    Path((package_id)): Path<(String)>,
    Query(query): Query<PackageQuery>,
) -> Result<Response> {
    let package = repository.get_package_by_id(&package_id).await?;

    if htmx {
        return Ok(GetPackageCardResponse {
            package
        }.into_response());
    }

    let version = query.version.map(|v| {
        let version = semver::Version::parse(&v).ok()?;
        package.versions.iter().find(|v| v.version == version)
    }).flatten().or(package.versions.first());

    Ok(GetPackageResponse {
        package: &package,
        version
    }.into_response())
}