use axum::Extension;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Result, IntoResponse, Response};
use serde::Deserialize;
use tantivy::collector::TopDocs;
use tantivy::schema::Value;
use crate::repository::model::Package;
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
        let schema = repository.index.schema();

        let id = schema.get_field("id").unwrap();
        let name = schema.get_field("name").unwrap();
        let version = schema.get_field("version").unwrap();
        let short_description = schema.get_field("short_description").unwrap();
        let readme = schema.get_field("readme").unwrap();
        let eeprom_name = schema.get_field("eeprom_name").unwrap();
        let eeprom_title = schema.get_field("eeprom_title").unwrap();
        let eeprom_description = schema.get_field("eeprom_description").unwrap();

        let searcher = repository.reader.searcher();

        let query_parser = tantivy::query::QueryParser::for_index(&repository.index, vec![id, name, version, short_description, readme, eeprom_name, eeprom_title, eeprom_description]);

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
            doc.get_first(id).map(|v| v.as_str()).flatten().map(|v| v.to_string())
        }).flatten().collect())
    }().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(ListPackageResponse {
        package_ids,
    })
}

pub async fn get_package(
    Extension(htmx): Extension<bool>,
    State(repository): State<Repository>,
    Path((package_id)): Path<(String)>,
) -> Result<Response> {
    let mut package: Package = match read_file_or_url(&repository.path(&format!("/Packages/{package_id}/metadata.toml"))).await.ok_or(StatusCode::NOT_FOUND)? {
        URLOrFile::URL(content) => std::str::from_utf8(&content).ok().map(|s| toml::from_str(s).unwrap()).flatten(),
        URLOrFile::File(file) => std::io::read_to_string(file).ok().map(|s| {
            toml::from_str(&s).unwrap()
        }).flatten(),
    }.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    package.id = package_id.clone();

    if htmx {
        return Ok(GetPackageCardResponse {
            package
        }.into_response());
    }

    let mut readme = read_file_or_url(&repository.path(&format!("/Packages/{package_id}/README.md"))).await;
    let mut readme_asciidoc = false;
    if readme.is_none() {
        readme_asciidoc = true;
        readme = read_file_or_url(&repository.path(&format!("/Packages/{package_id}/README.adoc"))).await;
    }
    package.readme = match readme.ok_or(StatusCode::NOT_FOUND)? {
        URLOrFile::URL(content) => std::str::from_utf8(&content).map(|s| s.to_string()).ok(),
        URLOrFile::File(file) => std::io::read_to_string(file).ok(),
    }.unwrap_or("".to_string());

    Ok(GetPackageResponse {
        package,
        readme_asciidoc,
    }.into_response())
}