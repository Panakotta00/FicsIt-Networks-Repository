use askama::Template;
use ficsit_networks_repository::model::*;

#[derive(Template)]
#[template(path = "package/list.html")]
pub struct ListPackageResponse {
    pub package_ids: Vec<String>,
}

#[derive(Template)]
#[template(path = "package/card.html")]
pub struct GetPackageCardResponse {
    pub package: Package,
}

#[derive(Template)]
#[template(path = "package/package.html")]
pub struct GetPackageResponse<'a, 'b: 'a> {
    pub package: &'a Package,
    pub version: Option<&'b Version>,
}
