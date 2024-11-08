use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Target {
    pub name: String,
    pub repository: String,
    pub branch: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Targets {
    pub targets: Vec<Target>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddTargetRequest {
    pub repository: String,
    pub branch: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildRequest {
    pub repository: String,
    pub branch: String,
}