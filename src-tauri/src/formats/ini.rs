use super::adapter::FormatAdapter;
use super::capabilities::FormatCapabilities;
use crate::document::{DocumentNode, NodeKind};
use crate::parsers::ini::{check, simple_fix};
use crate::FormatError;

pub struct IniAdapter;

impl FormatAdapter for IniAdapter {
    fn id(&self) -> &'static str {
        "ini"
    }

    fn name(&self) -> &'static str {
        "INI"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &[".ini", ".cfg", ".conf"]
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
        let has_semi_comment = lines.iter().any(|l| l.trim().starts_with(';'));
        if has_semi_comment && (has_section || trimmed.contains('=')) {
            return 92;
        }
        if has_section && looks_like_ini(trimmed) {
            return 91;
        }
        if has_section && lines.iter().any(|l| l.contains('=')) {
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
        let doc = self.parse(content)?;
        self.serialize(&doc)
    }

    fn parse(&self, content: &str) -> Result<DocumentNode, FormatError> {
        let mut root = DocumentNode::new(NodeKind::Object, "/");
        let mut sections: Vec<DocumentNode> = Vec::new();
        let mut current_section = DocumentNode::new(NodeKind::Section, "/default");
        current_section.key = Some("default".into());
        current_section.children = Some(Vec::new());

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
                continue;
            }
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let sec_name = trimmed[1..trimmed.len() - 1].trim();
                if let Some(children) = &current_section.children {
                    if !children.is_empty() || current_section.key.as_deref() != Some("default") {
                        sections.push(current_section);
                    }
                }
                current_section = DocumentNode::new(NodeKind::Section, format!("/{}", sec_name));
                current_section.key = Some(sec_name.to_string());
                current_section.children = Some(Vec::new());
            } else if let Some(eq_idx) = trimmed.find('=') {
                let key = trimmed[..eq_idx].trim();
                let value = trimmed[eq_idx + 1..].trim();
                let mut kv_node = DocumentNode::new(NodeKind::KeyValue, format!("/kv/{}", key));
                kv_node.key = Some(key.to_string());
                kv_node.value = Some(serde_json::Value::String(value.to_string()));
                if let Some(children) = &mut current_section.children {
                    children.push(kv_node);
                }
            }
        }
        sections.push(current_section);
        root.children = Some(sections);
        root.refresh_ids_and_paths();
        Ok(root)
    }

    fn serialize(&self, document: &DocumentNode) -> Result<String, FormatError> {
        let mut out = String::new();
        if let Some(sections) = &document.children {
            for (s_idx, sec) in sections.iter().enumerate() {
                let sec_name = sec.key.as_deref().unwrap_or("default");
                if sec_name != "default" {
                    if s_idx > 0 {
                        out.push('\n');
                    }
                    out.push('[');
                    out.push_str(sec_name);
                    out.push_str("]\n");
                }
                if let Some(kvs) = &sec.children {
                    for kv in kvs {
                        if let Some(k) = &kv.key {
                            let v_str = match &kv.value {
                                Some(serde_json::Value::String(s)) => s.clone(),
                                Some(v) => v.to_string(),
                                None => String::new(),
                            };
                            out.push_str(k);
                            out.push_str(" = ");
                            out.push_str(&v_str);
                            out.push('\n');
                        }
                    }
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

pub fn looks_like_ini(content: &str) -> bool {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with(';') {
            return true;
        }
        let Some(eq_pos) = trimmed.find('=') else {
            continue;
        };
        let value = trimmed[eq_pos + 1..].trim();
        if value.is_empty() {
            return true;
        }
        if is_ini_bare_value(value) {
            return true;
        }
    }
    false
}

pub fn is_ini_bare_value(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "true" | "false" | "nan" | "inf" | "+inf" | "-inf"
    ) {
        return false;
    }
    if value.starts_with('"')
        || value.starts_with('\'')
        || value.starts_with('[')
        || value.starts_with('{')
        || value.starts_with('+')
        || value.starts_with('-')
        || value.chars().next().is_some_and(|c| c.is_ascii_digit())
    {
        return false;
    }
    value.chars().any(|c| c.is_alphabetic())
}
