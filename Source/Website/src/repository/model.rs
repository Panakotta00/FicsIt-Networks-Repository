use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Package {
    #[serde(skip_deserialize)]
    pub id: Option<String>,
    pub name: String,
    pub short_description: String,
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
