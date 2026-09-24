use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatCapabilities {
    pub supports_repair: bool,
    pub supports_format: bool,
    pub supports_tree_editor: bool,
    pub supports_grid_editor: bool,
    pub supports_kv_editor: bool,
    pub supports_dom_editor: bool,
    pub supports_schema: bool,
    pub preserve_comments: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatDescriptor {
    pub id: String,
    pub name: String,
    pub extensions: Vec<String>,
    pub capabilities: FormatCapabilities,
}
