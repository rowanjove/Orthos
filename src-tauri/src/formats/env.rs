use super::adapter::FormatAdapter;
use super::capabilities::FormatCapabilities;
use crate::document::{DocumentNode, NodeKind};
use crate::parsers::env::{check, simple_fix};
use crate::FormatError;

pub struct EnvAdapter;

impl FormatAdapter for EnvAdapter {
    fn id(&self) -> &'static str {
        "env"
    }

    fn name(&self) -> &'static str {
        "ENV"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &[".env", ".env.local", ".env.development", ".env.production"]
    }

    fn sniff(&self, content: &str) -> u8 {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return 0;
        }
        let lines: Vec<&str> = trimmed.lines().collect();
        let has_section = lines.iter().any(|l| {
            let t = l.trim();
            t.starts_with('[') && t.ends_with(']')
        });
        if has_section {
            return 0;
        }
        let has_export = lines.iter().any(|l| l.trim().starts_with("export "));
        let has_env_assign = lines.iter().any(|l| {
            let t = l.trim();
            !t.starts_with('#') && t.contains('=') && !t.contains('{') && !t.contains('<')
        });
        if has_export {
            return 95;
        }
        if has_env_assign {
            return 75;
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
        let doc = self.parse(content)?;
        self.serialize(&doc)
    }

    fn parse(&self, content: &str) -> Result<DocumentNode, FormatError> {
        let mut root = DocumentNode::new(NodeKind::Object, "/");
        let mut kvs: Vec<DocumentNode> = Vec::new();

        for line in content.lines() {
            let mut trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("export ") {
                trimmed = rest.trim();
            }
            if let Some(eq_idx) = trimmed.find('=') {
                let key = trimmed[..eq_idx].trim();
                let mut value = trimmed[eq_idx + 1..].trim();
                if ((value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\'')))
                    && value.len() >= 2
                {
                    value = &value[1..value.len() - 1];
                }
                let mut node = DocumentNode::new(NodeKind::KeyValue, format!("/{}", key));
                node.key = Some(key.to_string());
                node.value = Some(serde_json::Value::String(value.to_string()));
                kvs.push(node);
            }
        }

        root.children = Some(kvs);
        root.refresh_ids_and_paths();
        Ok(root)
    }

    fn serialize(&self, document: &DocumentNode) -> Result<String, FormatError> {
        let mut out = String::new();
        if let Some(children) = &document.children {
            for child in children {
                if let Some(k) = &child.key {
                    let v_str = match &child.value {
                        Some(serde_json::Value::String(s)) => s.clone(),
                        Some(v) => v.to_string(),
                        None => String::new(),
                    };
                    out.push_str(k);
                    out.push('=');
                    if v_str.contains(' ') || v_str.contains('\n') || v_str.contains('#') {
                        out.push('"');
                        out.push_str(&v_str.replace('"', "\\\""));
                        out.push('"');
                    } else {
                        out.push_str(&v_str);
                    }
                    out.push('\n');
                }
            }
        }
        Ok(out)
    }

    fn capabilities(&self) -> FormatCapabilities {
        FormatCapabilities {
            supports_repair: true,
            supports_format: true,
            supports_tree_editor: true,
            supports_grid_editor: false,
            supports_kv_editor: true,
            supports_dom_editor: false,
            supports_schema: false,
            preserve_comments: false,
        }
    }
}
