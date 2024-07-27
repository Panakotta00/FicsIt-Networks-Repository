use askama::Template;
use crate::repository::model::Package;

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
pub struct GetPackageResponse {
    pub package: Package,
    pub readme_asciidoc: bool,
}
