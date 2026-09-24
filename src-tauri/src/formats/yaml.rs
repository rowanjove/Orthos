use super::adapter::FormatAdapter;
use super::capabilities::FormatCapabilities;
use crate::document::{json_value_to_node, node_to_json_value, DocumentNode};
use crate::parsers::yaml::{check, simple_fix};
use crate::FormatError;

pub struct YamlAdapter;

impl FormatAdapter for YamlAdapter {
    fn id(&self) -> &'static str {
        "yaml"
    }

    fn name(&self) -> &'static str {
        "YAML"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &[".yaml", ".yml"]
    }

    fn sniff(&self, content: &str) -> u8 {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return 0;
        }
        if trimmed.starts_with("---") || trimmed.contains("\n---\n") {
            return 90;
        }
        let lines: Vec<&str> = trimmed.lines().collect();
        let has_mapping = lines.iter().any(|l| {
            let t = l.trim();
            !t.starts_with('#') && t.contains(": ")
        });
        let has_seq = lines.iter().any(|l| l.trim().starts_with("- "));
        if has_mapping && (has_seq || lines.len() > 1) {
            return 80;
        }
        if has_mapping {
            return 60;
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
        let val: serde_yaml::Value = serde_yaml::from_str(content).map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "YAML 格式化失败：包含语法错误".into(),
        })?;
        serde_yaml::to_string(&val).map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "YAML 重新生成失败".into(),
        })
    }

    fn parse(&self, content: &str) -> Result<DocumentNode, FormatError> {
        let yaml_val: serde_yaml::Value =
            serde_yaml::from_str(content).map_err(|e| FormatError {
                line: None,
                col: None,
                near: None,
                raw: e.to_string(),
                friendly: "无法解析为 YAML 结构".into(),
            })?;
        let json_val = serde_json::to_value(yaml_val).map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "YAML 数据结构映射失败".into(),
        })?;
        let mut node = json_value_to_node(&json_val, None, "/");
        node.refresh_ids_and_paths();
        Ok(node)
    }

    fn serialize(&self, document: &DocumentNode) -> Result<String, FormatError> {
        let json_val = node_to_json_value(document)?;
        serde_yaml::to_string(&json_val).map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "序列化 YAML 失败".into(),
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
            supports_schema: false,
            preserve_comments: false,
        }
    }
}
