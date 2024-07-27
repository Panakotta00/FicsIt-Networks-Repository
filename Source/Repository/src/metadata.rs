use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub short_description: String,
    pub readme: String,
    pub authors: Vec<String>,
    #[serde(alias = "EEPROM")]
    pub eeprom: Vec<EEPROM>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EEPROM {
    pub name: String,
    pub title: String,
    pub description: String,
}
