use crate::document::DocumentNode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDiagnostic {
    pub path: Option<String>,
    pub message: String,
    pub severity: String, // "error" | "warning" | "info"
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDescriptor {
    pub id: String,
    pub name: String,
    pub description: String,
}

pub trait ConfigProfile: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn detect(&self, filename: &str, content: &str, document: &DocumentNode) -> u8;
    fn validate(&self, document: &DocumentNode) -> Vec<ProfileDiagnostic>;
}
