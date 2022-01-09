use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Dependencies(pub HashMap<String, String>);

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectConfiguration {
    pub name: String,
    pub version: String,
    pub description: String,
    pub compiler_version: String,
    pub dependencies: Dependencies,
}
