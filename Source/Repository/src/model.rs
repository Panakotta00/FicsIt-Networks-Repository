use std::fmt::{Display, Formatter, Write};

#[derive(Clone)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub short_description: String,
    pub readme: Readme,
    pub tags: Vec<String>,
    pub authors: Vec<String>,
    pub versions: Vec<Version>,
}

#[derive(Clone)]
pub enum Readme {
    ASCIIDOC(String),
    Markdown(String)
}

impl Display for Readme {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Readme::ASCIIDOC(s) => f.write_str(s),
            Readme::Markdown(s) => f.write_str(s),
        }
    }
}

#[derive(Clone)]
pub struct Version {
    pub version: semver::Version,
    pub fin_version: Option<semver::VersionReq>,
    pub game_version: Option<semver::VersionReq>,
    pub mod_dependencies: Vec<ModDependency>,
    pub eeprom: Vec<EEPROM>,
}

#[derive(Clone)]
pub struct ModDependency {
    pub id: String,
    pub version: Option<semver::VersionReq>,
}

#[derive(Clone)]
pub struct EEPROM {
    pub name: String,
    pub title: String,
    pub description: String,
}

impl Package {
    pub fn from_metadata(id: String, readme: Readme, versions: Vec<Version>, metadata: crate::metadata::Package) -> Self {
        Package {
            id,
            name: metadata.name,
            short_description: metadata.short_description,
            readme,
            tags: metadata.tags,
            authors: metadata.authors,
            versions,
        }
    }
}