pub mod package;

use askama::Template;

#[derive(Template)]
#[template(source = "", ext = "")]
pub struct EmptyResponse {}

#[derive(Template)]
#[template(path = "index.html")]
pub struct GetIndexResponse;
