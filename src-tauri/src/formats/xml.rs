use super::adapter::FormatAdapter;
use super::capabilities::FormatCapabilities;
use crate::document::{DocumentNode, NodeKind};
use crate::parsers::xml::{check, simple_fix};
use crate::FormatError;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;

pub struct XmlAdapter;

impl FormatAdapter for XmlAdapter {
    fn id(&self) -> &'static str {
        "xml"
    }

    fn name(&self) -> &'static str {
        "XML"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &[".xml", ".svg", ".plist"]
    }

    fn sniff(&self, content: &str) -> u8 {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return 0;
        }
        if trimmed.starts_with("<?xml") {
            return 99;
        }
        if trimmed.starts_with('<') && trimmed.contains('>') {
            return 85;
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
        let mut reader = Reader::from_str(content.trim());
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut stack: Vec<DocumentNode> = Vec::new();
        let mut root_node: Option<DocumentNode> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let mut node = DocumentNode::new(NodeKind::Element, "/");
                    node.key = Some(tag);
                    let mut attrs = HashMap::new();
                    for attr in e.attributes().flatten() {
                        let k = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let v = String::from_utf8_lossy(&attr.value).to_string();
                        attrs.insert(k, v);
                    }
                    if !attrs.is_empty() {
                        node.metadata = Some(attrs);
                    }
                    node.children = Some(Vec::new());
                    stack.push(node);
                }
                Ok(Event::Empty(e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let mut node = DocumentNode::new(NodeKind::Element, "/");
                    node.key = Some(tag);
                    let mut attrs = HashMap::new();
                    for attr in e.attributes().flatten() {
                        let k = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let v = String::from_utf8_lossy(&attr.value).to_string();
                        attrs.insert(k, v);
                    }
                    if !attrs.is_empty() {
                        node.metadata = Some(attrs);
                    }

                    if let Some(parent) = stack.last_mut() {
                        if parent.children.is_none() {
                            parent.children = Some(Vec::new());
                        }
                        parent.children.as_mut().unwrap().push(node);
                    } else if root_node.is_none() {
                        root_node = Some(node);
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape().map(|t| t.to_string()).unwrap_or_default();
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        if let Some(cur) = stack.last_mut() {
                            cur.value = Some(serde_json::Value::String(trimmed.to_string()));
                        }
                    }
                }
                Ok(Event::CData(e)) => {
                    let text = String::from_utf8_lossy(&e).to_string();
                    if let Some(cur) = stack.last_mut() {
                        cur.value = Some(serde_json::Value::String(text));
                    }
                }
                Ok(Event::End(_)) => {
                    if let Some(node) = stack.pop() {
                        if let Some(parent) = stack.last_mut() {
                            if parent.children.is_none() {
                                parent.children = Some(Vec::new());
                            }
                            parent.children.as_mut().unwrap().push(node);
                        } else {
                            root_node = Some(node);
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(FormatError {
                        line: None,
                        col: None,
                        near: None,
                        raw: e.to_string(),
                        friendly: "XML 解析错误".into(),
                    });
                }
                _ => {}
            }
            buf.clear();
        }

        let mut root = root_node.unwrap_or_else(|| {
            let mut r = DocumentNode::new(NodeKind::Element, "/root");
            r.key = Some("root".into());
            r
        });
        root.refresh_ids_and_paths();
        Ok(root)
    }

    fn serialize(&self, document: &DocumentNode) -> Result<String, FormatError> {
        let mut out = String::new();
        serialize_xml_node(document, &mut out, 0);
        Ok(out)
    }

    fn capabilities(&self) -> FormatCapabilities {
        FormatCapabilities {
            supports_repair: true,
            supports_format: true,
            supports_tree_editor: true,
            supports_grid_editor: false,
            supports_kv_editor: false,
            supports_dom_editor: true,
            supports_schema: false,
            preserve_comments: false,
        }
    }
}

fn serialize_xml_node(node: &DocumentNode, out: &mut String, indent: usize) {
    let tag = node.key.as_deref().unwrap_or("element");
    let indent_str = "  ".repeat(indent);
    out.push_str(&indent_str);
    out.push('<');
    out.push_str(tag);

    if let Some(meta) = &node.metadata {
        for (k, v) in meta {
            out.push(' ');
            out.push_str(k);
            out.push_str("=\"");
            out.push_str(&v.replace('&', "&amp;").replace('"', "&quot;"));
            out.push('"');
        }
    }

    let has_children = node
        .children
        .as_ref()
        .map(|c| !c.is_empty())
        .unwrap_or(false);
    let has_text = node.value.as_ref().is_some();

    if !has_children && !has_text {
        out.push_str(" />\n");
        return;
    }

    out.push('>');

    if let Some(val) = &node.value {
        let text = match val {
            serde_json::Value::String(s) => s.as_str(),
            _ => &val.to_string(),
        };
        out.push_str(
            &text
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;"),
        );
    }

    if has_children {
        out.push('\n');
        if let Some(children) = &node.children {
            for child in children {
                serialize_xml_node(child, out, indent + 1);
            }
        }
        out.push_str(&indent_str);
    }

    out.push_str("</");
    out.push_str(tag);
    out.push_str(">\n");
}
