use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::IntoResponse,
};

pub async fn get_htmx_header(
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let htmx = headers.contains_key("HX-Request");
    req.extensions_mut().insert(htmx);

    Ok(next.run(req).await)
}
