use super::adapter::FormatAdapter;
use super::capabilities::FormatCapabilities;
use super::ini::looks_like_ini;
use crate::document::{json_value_to_node, node_to_json_value, DocumentNode};
use crate::parsers::toml::{check, simple_fix};
use crate::FormatError;

pub struct TomlAdapter;

impl FormatAdapter for TomlAdapter {
    fn id(&self) -> &'static str {
        "toml"
    }

    fn name(&self) -> &'static str {
        "TOML"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &[".toml"]
    }

    fn sniff(&self, content: &str) -> u8 {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return 0;
        }
        if looks_like_ini(trimmed) {
            return 0;
        }
        if toml::from_str::<toml::Value>(trimmed).is_ok() && trimmed.contains('=') {
            return 95;
        }
        let lines: Vec<&str> = trimmed.lines().collect();
        let has_section = lines.iter().any(|l| {
            let t = l.trim();
            t.starts_with('[') && t.ends_with(']')
        });
        let has_assign = lines.iter().any(|l| l.contains('='));
        if has_section && has_assign {
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
        let val: toml::Value = toml::from_str(content).map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "TOML 格式化失败：包含语法错误".into(),
        })?;
        toml::to_string_pretty(&val).map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "TOML 重新生成失败".into(),
        })
    }

    fn parse(&self, content: &str) -> Result<DocumentNode, FormatError> {
        let toml_val: toml::Value = toml::from_str(content).map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "无法解析为 TOML 结构".into(),
        })?;
        let json_val = serde_json::to_value(toml_val).map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "TOML 数据映射失败".into(),
        })?;
        let mut node = json_value_to_node(&json_val, None, "/");
        node.refresh_ids_and_paths();
        Ok(node)
    }

    fn serialize(&self, document: &DocumentNode) -> Result<String, FormatError> {
        let json_val = node_to_json_value(document)?;
        let toml_val: toml::Value = serde_json::from_value(json_val).map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "转换 TOML 失败".into(),
        })?;
        toml::to_string_pretty(&toml_val).map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "序列化 TOML 失败".into(),
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
