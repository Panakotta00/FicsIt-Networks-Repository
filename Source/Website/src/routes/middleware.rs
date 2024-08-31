use axum::http::Uri;
use axum::{
	extract::Request,
	http,
	http::{HeaderMap, StatusCode},
	middleware::Next,
	response::IntoResponse,
};
use std::str::FromStr;

pub type HTMXExtension = Option<Uri>;

pub async fn get_htmx_header(
	headers: HeaderMap,
	mut req: Request,
	next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let htmx = headers.contains_key("HX-Request");
	req.extensions_mut().insert(
		htmx.then(|| {
			headers
				.get("HX-Current-URL")
				.map(|s| s.to_str().ok())
				.flatten()
				.map(|s| http::uri::Uri::from_str(s).ok())
				.flatten()
		})
		.flatten(),
	);

	Ok(next.run(req).await)
}

#[derive(Clone)]
pub struct AcceptJsonOnly(pub bool);

pub async fn accept_json_only(
	headers: HeaderMap,
	mut req: Request,
	next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let json = headers.get("Accept")
		.map(|h| h.to_str().ok() == Some("application/json"));
	req.extensions_mut().insert(AcceptJsonOnly(json.unwrap_or(false)));

	Ok(next.run(req).await)
}

