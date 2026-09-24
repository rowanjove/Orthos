use crate::FormatError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
    Object,
    Array,
    String,
    Number,
    Boolean,
    Null,
    Section,
    KeyValue,
    Table,
    Row,
    Element,
    Attribute,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentNode {
    pub id: String,
    pub kind: NodeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<DocumentNode>>,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

impl DocumentNode {
    pub fn new(kind: NodeKind, path: impl Into<String>) -> Self {
        Self {
            id: String::new(),
            kind,
            key: None,
            value: None,
            children: None,
            path: path.into(),
            metadata: None,
        }
    }

    /// 重新为整棵树分配唯一 ID 与规范化路径
    pub fn refresh_ids_and_paths(&mut self) {
        let mut counter = 0usize;
        self.assign_internal("/", &mut counter);
    }

    fn assign_internal(&mut self, current_path: &str, counter: &mut usize) {
        *counter += 1;
        self.id = format!("node-{}", *counter);
        self.path = current_path.to_string();

        if let Some(children) = &mut self.children {
            let mut key_counts: HashMap<String, usize> = HashMap::new();
            for child in children.iter() {
                if let Some(k) = &child.key {
                    *key_counts.entry(k.clone()).or_insert(0) += 1;
                }
            }

            let mut key_indices: HashMap<String, usize> = HashMap::new();
            for (index, child) in children.iter_mut().enumerate() {
                let child_path = if current_path == "/" {
                    if let Some(k) = &child.key {
                        if *key_counts.get(k).unwrap_or(&0) > 1 {
                            let idx = key_indices.entry(k.clone()).or_insert(0);
                            let p = format!("/{}#{}", k, idx);
                            *idx += 1;
                            p
                        } else {
                            format!("/{}", k)
                        }
                    } else {
                        format!("/{}", index)
                    }
                } else if let Some(k) = &child.key {
                    if *key_counts.get(k).unwrap_or(&0) > 1 {
                        let idx = key_indices.entry(k.clone()).or_insert(0);
                        let p = format!("{}/{}#{}", current_path, k, idx);
                        *idx += 1;
                        p
                    } else {
                        format!("{}/{}", current_path, k)
                    }
                } else {
                    format!("{}/{}", current_path, index)
                };
                child.assign_internal(&child_path, counter);
            }
        }
    }

    /// 根据路径查找不可变节点
    pub fn find_by_path(&self, target_path: &str) -> Option<&DocumentNode> {
        let target = normalize_path(target_path);
        let self_norm = normalize_path(&self.path);
        if self_norm == target {
            return Some(self);
        }
        if !target.contains('#') && self_norm.ends_with("#0") {
            let prefix = &self_norm[..self_norm.len() - 2];
            if prefix == target {
                return Some(self);
            }
        }
        if let Some(children) = &self.children {
            for child in children {
                if let Some(found) = child.find_by_path(target_path) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// 根据路径查找可变节点引用
    pub fn find_by_path_mut(&mut self, target_path: &str) -> Option<&mut DocumentNode> {
        let target = normalize_path(target_path);
        let self_norm = normalize_path(&self.path);
        if self_norm == target {
            return Some(self);
        }
        if !target.contains('#') && self_norm.ends_with("#0") {
            let prefix = &self_norm[..self_norm.len() - 2];
            if prefix == target {
                return Some(self);
            }
        }
        if let Some(children) = &mut self.children {
            for child in children {
                if let Some(found) = child.find_by_path_mut(target_path) {
                    return Some(found);
                }
            }
        }
        None
    }
}

pub fn normalize_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed == "/" {
        "/".to_string()
    } else if !trimmed.starts_with('/') {
        format!("/{}", trimmed)
    } else {
        trimmed.to_string()
    }
}

/// JSON Value 转 DocumentNode
pub fn json_value_to_node(
    value: &serde_json::Value,
    key: Option<String>,
    path: &str,
) -> DocumentNode {
    match value {
        serde_json::Value::Object(map) => {
            let mut node = DocumentNode::new(NodeKind::Object, path);
            node.key = key;
            let mut children = Vec::with_capacity(map.len());
            for (k, v) in map {
                let child_path = if path == "/" {
                    format!("/{}", k)
                } else {
                    format!("{}/{}", path, k)
                };
                children.push(json_value_to_node(v, Some(k.clone()), &child_path));
            }
            node.children = Some(children);
            node
        }
        serde_json::Value::Array(arr) => {
            let mut node = DocumentNode::new(NodeKind::Array, path);
            node.key = key;
            let mut children = Vec::with_capacity(arr.len());
            for (i, v) in arr.iter().enumerate() {
                let child_path = if path == "/" {
                    format!("/{}", i)
                } else {
                    format!("{}/{}", path, i)
                };
                children.push(json_value_to_node(v, None, &child_path));
            }
            node.children = Some(children);
            node
        }
        serde_json::Value::String(s) => {
            let mut node = DocumentNode::new(NodeKind::String, path);
            node.key = key;
            node.value = Some(serde_json::Value::String(s.clone()));
            node
        }
        serde_json::Value::Number(n) => {
            let mut node = DocumentNode::new(NodeKind::Number, path);
            node.key = key;
            node.value = Some(serde_json::Value::Number(n.clone()));
            node
        }
        serde_json::Value::Bool(b) => {
            let mut node = DocumentNode::new(NodeKind::Boolean, path);
            node.key = key;
            node.value = Some(serde_json::Value::Bool(*b));
            node
        }
        serde_json::Value::Null => {
            let mut node = DocumentNode::new(NodeKind::Null, path);
            node.key = key;
            node.value = Some(serde_json::Value::Null);
            node
        }
    }
}

/// DocumentNode 转 JSON Value
pub fn node_to_json_value(node: &DocumentNode) -> Result<serde_json::Value, FormatError> {
    match node.kind {
        NodeKind::Object | NodeKind::Section => {
            let mut map = serde_json::Map::new();
            if let Some(children) = &node.children {
                for (index, child) in children.iter().enumerate() {
                    let key = child
                        .key
                        .clone()
                        .unwrap_or_else(|| format!("item_{}", index));
                    map.insert(key, node_to_json_value(child)?);
                }
            }
            Ok(serde_json::Value::Object(map))
        }
        NodeKind::Array => {
            let mut arr = Vec::new();
            if let Some(children) = &node.children {
                for child in children {
                    arr.push(node_to_json_value(child)?);
                }
            }
            Ok(serde_json::Value::Array(arr))
        }
        NodeKind::String => {
            let s = match &node.value {
                Some(serde_json::Value::String(val)) => val.clone(),
                Some(val) => val.to_string(),
                None => String::new(),
            };
            Ok(serde_json::Value::String(s))
        }
        NodeKind::Number => {
            if let Some(val) = &node.value {
                if val.is_number() {
                    return Ok(val.clone());
                }
                if let Some(s) = val.as_str() {
                    if let Ok(i) = s.parse::<i64>() {
                        return Ok(serde_json::Value::Number(i.into()));
                    }
                    if let Ok(f) = s.parse::<f64>() {
                        if let Some(num) = serde_json::Number::from_f64(f) {
                            return Ok(serde_json::Value::Number(num));
                        }
                    }
                }
            }
            Ok(serde_json::Value::Number(0.into()))
        }
        NodeKind::Boolean => {
            let b = match &node.value {
                Some(serde_json::Value::Bool(b)) => *b,
                Some(serde_json::Value::String(s)) => s.eq_ignore_ascii_case("true"),
                _ => false,
            };
            Ok(serde_json::Value::Bool(b))
        }
        NodeKind::Null => Ok(serde_json::Value::Null),
        NodeKind::KeyValue => {
            let v = node
                .value
                .clone()
                .unwrap_or_else(|| serde_json::Value::String(String::new()));
            Ok(v)
        }
        NodeKind::Table => {
            let mut rows = Vec::new();
            if let Some(children) = &node.children {
                for child in children {
                    rows.push(node_to_json_value(child)?);
                }
            }
            Ok(serde_json::Value::Array(rows))
        }
        NodeKind::Row => {
            let mut cells = Vec::new();
            if let Some(children) = &node.children {
                for child in children {
                    cells.push(node_to_json_value(child)?);
                }
            }
            Ok(serde_json::Value::Array(cells))
        }
        NodeKind::Element => {
            let mut map = serde_json::Map::new();
            if let Some(k) = &node.key {
                map.insert("_tag".into(), serde_json::Value::String(k.clone()));
            }
            if let Some(meta) = &node.metadata {
                let mut attrs = serde_json::Map::new();
                for (mk, mv) in meta {
                    attrs.insert(mk.clone(), serde_json::Value::String(mv.clone()));
                }
                map.insert("_attributes".into(), serde_json::Value::Object(attrs));
            }
            if let Some(val) = &node.value {
                map.insert("_text".into(), val.clone());
            }
            if let Some(children) = &node.children {
                let mut c_arr = Vec::new();
                for c in children {
                    c_arr.push(node_to_json_value(c)?);
                }
                map.insert("_children".into(), serde_json::Value::Array(c_arr));
            }
            Ok(serde_json::Value::Object(map))
        }
        NodeKind::Attribute => {
            let v = node
                .value
                .clone()
                .unwrap_or_else(|| serde_json::Value::String(String::new()));
            Ok(v)
        }
    }
}
