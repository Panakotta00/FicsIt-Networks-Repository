use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Package {
    #[serde(default = "String::new")]
    pub id: String,
    pub name: String,
    pub short_description: String,
    #[serde(default = "String::new")]
    pub readme: String,
    #[serde(alias = "EEPROM")]
    pub eeprom: Vec<EEPROMMetadata>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EEPROMMetadata {
    pub name: String,
    pub title: String,
    pub description: String,
}
