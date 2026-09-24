use super::node::{json_value_to_node, normalize_path, DocumentNode, NodeKind};
use crate::FormatError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DocumentPatch {
    SetValue {
        path: String,
        value: serde_json::Value,
    },
    SetKey {
        path: String,
        new_key: String,
    },
    AddNode {
        parent_path: String,
        index: Option<usize>,
        key: Option<String>,
        value: Option<serde_json::Value>,
        kind: NodeKind,
    },
    RemoveNode {
        path: String,
    },
    SetType {
        path: String,
        target_type: String,
    },
    AddRow {
        parent_path: String,
        index: Option<usize>,
        values: Vec<String>,
    },
    RemoveRow {
        parent_path: String,
        index: usize,
    },
    AddColumn {
        name: String,
        default_value: String,
    },
    RemoveColumn {
        name: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchResult {
    pub content: String,
    pub node: DocumentNode,
    pub valid: bool,
    pub errors: Vec<FormatError>,
}

/// 对 DocumentNode 施加 patch
pub fn apply_patch_to_node(root: &mut DocumentNode, patch: &DocumentPatch) -> Result<(), String> {
    match patch {
        DocumentPatch::SetValue { path, value } => {
            let target_path = normalize_path(path);
            let node = root
                .find_by_path_mut(&target_path)
                .ok_or_else(|| format!("未找到路径对应节点: {}", target_path))?;

            match value {
                serde_json::Value::Null => {
                    node.kind = NodeKind::Null;
                    node.value = Some(serde_json::Value::Null);
                }
                serde_json::Value::Bool(b) => {
                    node.kind = NodeKind::Boolean;
                    node.value = Some(serde_json::Value::Bool(*b));
                }
                serde_json::Value::Number(n) => {
                    node.kind = NodeKind::Number;
                    node.value = Some(serde_json::Value::Number(n.clone()));
                }
                serde_json::Value::String(s) => {
                    node.kind = NodeKind::String;
                    node.value = Some(serde_json::Value::String(s.clone()));
                }
                serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                    let key = node.key.clone();
                    let new_node = json_value_to_node(value, key, &target_path);
                    *node = new_node;
                }
            }
        }
        DocumentPatch::SetKey { path, new_key } => {
            let target_path = normalize_path(path);
            let node = root
                .find_by_path_mut(&target_path)
                .ok_or_else(|| format!("未找到路径对应节点: {}", target_path))?;
            node.key = Some(new_key.clone());
        }
        DocumentPatch::AddNode {
            parent_path,
            index,
            key,
            value,
            kind,
        } => {
            let parent_norm = normalize_path(parent_path);
            let parent = root
                .find_by_path_mut(&parent_norm)
                .ok_or_else(|| format!("未找到父路径对应节点: {}", parent_norm))?;

            if parent.children.is_none() {
                parent.children = Some(Vec::new());
            }

            let mut new_node = DocumentNode::new(kind.clone(), "/tmp");
            new_node.key = key.clone();
            new_node.value = value.clone();
            if matches!(
                kind,
                NodeKind::Object | NodeKind::Array | NodeKind::Section | NodeKind::Table
            ) {
                new_node.children = Some(Vec::new());
            }

            let children = parent.children.as_mut().unwrap();
            if let Some(idx) = index {
                let insert_at = std::cmp::min(*idx, children.len());
                children.insert(insert_at, new_node);
            } else {
                children.push(new_node);
            }
        }
        DocumentPatch::RemoveNode { path } => {
            let target_path = normalize_path(path);
            if target_path == "/" {
                return Err("不能删除根节点".into());
            }
            let (parent_path, _) = split_parent_and_leaf(&target_path)?;
            let parent = root
                .find_by_path_mut(&parent_path)
                .ok_or_else(|| format!("未找到父节点: {}", parent_path))?;

            if let Some(children) = &mut parent.children {
                children.retain(|c| normalize_path(&c.path) != target_path);
            }
        }
        DocumentPatch::SetType { path, target_type } => {
            let target_path = normalize_path(path);
            let node = root
                .find_by_path_mut(&target_path)
                .ok_or_else(|| format!("未找到路径对应节点: {}", target_path))?;

            match target_type.to_lowercase().as_str() {
                "string" => {
                    let cur = node
                        .value
                        .as_ref()
                        .map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            _ => v.to_string(),
                        })
                        .unwrap_or_default();
                    node.kind = NodeKind::String;
                    node.value = Some(serde_json::Value::String(cur));
                    node.children = None;
                }
                "number" => {
                    let cur = node
                        .value
                        .as_ref()
                        .and_then(|v| match v {
                            serde_json::Value::Number(n) => Some(n.clone()),
                            serde_json::Value::String(s) => s.parse::<i64>().ok().map(|i| i.into()),
                            _ => None,
                        })
                        .unwrap_or_else(|| 0.into());
                    node.kind = NodeKind::Number;
                    node.value = Some(serde_json::Value::Number(cur));
                    node.children = None;
                }
                "boolean" => {
                    let cur = node
                        .value
                        .as_ref()
                        .map(|v| match v {
                            serde_json::Value::Bool(b) => *b,
                            serde_json::Value::String(s) => s.eq_ignore_ascii_case("true"),
                            _ => false,
                        })
                        .unwrap_or(false);
                    node.kind = NodeKind::Boolean;
                    node.value = Some(serde_json::Value::Bool(cur));
                    node.children = None;
                }
                "null" => {
                    node.kind = NodeKind::Null;
                    node.value = Some(serde_json::Value::Null);
                    node.children = None;
                }
                "object" => {
                    node.kind = NodeKind::Object;
                    node.value = None;
                    if node.children.is_none() {
                        node.children = Some(Vec::new());
                    }
                }
                "array" => {
                    node.kind = NodeKind::Array;
                    node.value = None;
                    if node.children.is_none() {
                        node.children = Some(Vec::new());
                    }
                }
                _ => return Err(format!("不支持的目标类型: {}", target_type)),
            }
        }
        DocumentPatch::AddRow {
            parent_path,
            index,
            values,
        } => {
            let parent_norm = normalize_path(parent_path);
            let parent = root
                .find_by_path_mut(&parent_norm)
                .ok_or_else(|| format!("未找到父节点: {}", parent_norm))?;

            let mut row_node = DocumentNode::new(NodeKind::Row, "/tmp/row");
            let mut cells = Vec::new();
            for val in values {
                let mut cell = DocumentNode::new(NodeKind::String, "/tmp/cell");
                cell.value = Some(serde_json::Value::String(val.clone()));
                cells.push(cell);
            }
            row_node.children = Some(cells);

            if parent.children.is_none() {
                parent.children = Some(Vec::new());
            }
            let children = parent.children.as_mut().unwrap();
            if let Some(idx) = index {
                let at = std::cmp::min(*idx, children.len());
                children.insert(at, row_node);
            } else {
                children.push(row_node);
            }
        }
        DocumentPatch::RemoveRow { parent_path, index } => {
            let parent_norm = normalize_path(parent_path);
            let parent = root
                .find_by_path_mut(&parent_norm)
                .ok_or_else(|| format!("未找到父节点: {}", parent_norm))?;
            if let Some(children) = &mut parent.children {
                if *index < children.len() {
                    children.remove(*index);
                }
            }
        }
        DocumentPatch::AddColumn {
            name,
            default_value,
        } => {
            if let Some(rows) = &mut root.children {
                for (r_idx, row) in rows.iter_mut().enumerate() {
                    if let Some(cells) = &mut row.children {
                        let mut cell = DocumentNode::new(NodeKind::String, "/tmp/cell");
                        if r_idx == 0 {
                            cell.value = Some(serde_json::Value::String(name.clone()));
                        } else {
                            cell.value = Some(serde_json::Value::String(default_value.clone()));
                        }
                        cells.push(cell);
                    }
                }
            }
        }
        DocumentPatch::RemoveColumn { name } => {
            let mut col_index = None;
            if let Some(rows) = &root.children {
                if let Some(first_row) = rows.first() {
                    if let Some(cells) = &first_row.children {
                        for (i, cell) in cells.iter().enumerate() {
                            if let Some(serde_json::Value::String(s)) = &cell.value {
                                if s == name {
                                    col_index = Some(i);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            if let Some(idx) = col_index {
                if let Some(rows) = &mut root.children {
                    for row in rows.iter_mut() {
                        if let Some(cells) = &mut row.children {
                            if idx < cells.len() {
                                cells.remove(idx);
                            }
                        }
                    }
                }
            }
        }
    }

    // 重新规整路径与 ID
    root.refresh_ids_and_paths();
    Ok(())
}

fn split_parent_and_leaf(path: &str) -> Result<(String, String), String> {
    let norm = normalize_path(path);
    if norm == "/" {
        return Err("根路径无父节点".into());
    }
    let last_slash = norm.rfind('/').unwrap();
    let parent = if last_slash == 0 {
        "/".to_string()
    } else {
        norm[..last_slash].to_string()
    };
    let leaf = norm[last_slash + 1..].to_string();
    Ok((parent, leaf))
}
