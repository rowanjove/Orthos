use super::adapter::FormatAdapter;
use super::capabilities::FormatCapabilities;
use crate::document::{json_value_to_node, node_to_json_value, DocumentNode};
use crate::parsers::json::{check, simple_fix};
use crate::FormatError;

pub struct JsonAdapter;

impl FormatAdapter for JsonAdapter {
    fn id(&self) -> &'static str {
        "json"
    }

    fn name(&self) -> &'static str {
        "JSON"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &[".json"]
    }

    fn sniff(&self, content: &str) -> u8 {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return 0;
        }
        if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
            return 98;
        }
        if trimmed.starts_with('{') {
            return 85;
        }
        if trimmed.starts_with('[') && !trimmed.lines().any(|l| l.trim().contains('=')) {
            return 80;
        }
        0
    }

    fn validate(&self, content: &str) -> Vec<FormatError> {
        check(content).0
    }

    fn repair(&self, content: &str) -> Option<String> {
        let fixed = simple_fix(content);
        if fixed != content {
            Some(fixed)
        } else {
            None
        }
    }

    fn format(&self, content: &str) -> Result<String, FormatError> {
        let val: serde_json::Value = serde_json::from_str(content.trim_start_matches('\u{feff}'))
            .map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "格式化失败：内容包含语法错误".into(),
        })?;
        serde_json::to_string_pretty(&val).map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "JSON 序列化失败".into(),
        })
    }

    fn parse(&self, content: &str) -> Result<DocumentNode, FormatError> {
        let val: serde_json::Value = serde_json::from_str(content.trim_start_matches('\u{feff}'))
            .map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "无法解析为 JSON 结构".into(),
        })?;
        let mut node = json_value_to_node(&val, None, "/");
        node.refresh_ids_and_paths();
        Ok(node)
    }

    fn serialize(&self, document: &DocumentNode) -> Result<String, FormatError> {
        let val = node_to_json_value(document)?;
        serde_json::to_string_pretty(&val).map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "生成 JSON 文本失败".into(),
        })
    }

    fn capabilities(&self) -> FormatCapabilities {
        FormatCapabilities {
            supports_repair: true,
            supports_format: true,
            supports_tree_editor: true,
            supports_grid_editor: false,
            supports_kv_editor: false,
            supports_dom_editor: false,
            supports_schema: true,
            preserve_comments: false,
        }
    }
}
