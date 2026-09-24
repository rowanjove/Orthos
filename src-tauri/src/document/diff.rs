use super::node::{DocumentNode, NodeKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticDiffItem {
    pub change_type: String, // "added" | "removed" | "modified"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff_type: Option<String>,
    pub path: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticDiffResult {
    pub items: Vec<SemanticDiffItem>,
    pub added_count: usize,
    pub removed_count: usize,
    pub modified_count: usize,
}

pub fn compute_semantic_diff(
    old_root: &DocumentNode,
    new_root: &DocumentNode,
) -> SemanticDiffResult {
    let mut old_map = HashMap::new();
    let mut new_map = HashMap::new();

    collect_leaf_values(old_root, &mut old_map);
    collect_leaf_values(new_root, &mut new_map);

    let mut items = Vec::new();
    let mut added_count = 0;
    let mut removed_count = 0;
    let mut modified_count = 0;

    // 检查修改和新增
    for (path, new_val) in &new_map {
        if let Some(old_val) = old_map.get(path) {
            if old_val != new_val {
                modified_count += 1;
                items.push(SemanticDiffItem {
                    change_type: "modified".into(),
                    diff_type: Some("modified".into()),
                    path: path.clone(),
                    old_value: Some(old_val.clone()),
                    new_value: Some(new_val.clone()),
                    description: format!("{}: '{}' → '{}'", path, old_val, new_val),
                });
            }
        } else {
            added_count += 1;
            items.push(SemanticDiffItem {
                change_type: "added".into(),
                diff_type: Some("added".into()),
                path: path.clone(),
                old_value: None,
                new_value: Some(new_val.clone()),
                description: format!("新增 {}: '{}'", path, new_val),
            });
        }
    }

    // 检查删除
    for (path, old_val) in &old_map {
        if !new_map.contains_key(path) {
            removed_count += 1;
            items.push(SemanticDiffItem {
                change_type: "removed".into(),
                diff_type: Some("removed".into()),
                path: path.clone(),
                old_value: Some(old_val.clone()),
                new_value: None,
                description: format!("删除 {}: 原值 '{}'", path, old_val),
            });
        }
    }

    // 按路径排序
    items.sort_by(|a, b| a.path.cmp(&b.path));

    SemanticDiffResult {
        items,
        added_count,
        removed_count,
        modified_count,
    }
}

fn collect_leaf_values(node: &DocumentNode, map: &mut HashMap<String, String>) {
    let is_leaf = match node.kind {
        NodeKind::String
        | NodeKind::Number
        | NodeKind::Boolean
        | NodeKind::Null
        | NodeKind::KeyValue => true,
        NodeKind::Element => node.children.as_ref().map(|c| c.is_empty()).unwrap_or(true),
        _ => node.children.as_ref().map(|c| c.is_empty()).unwrap_or(true),
    };

    if is_leaf && node.path != "/" {
        let val_str = match &node.value {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(v) => v.to_string(),
            None => "".into(),
        };
        map.insert(node.path.clone(), val_str);
    }

    // 收集 Element 节点的属性
    if let Some(meta) = &node.metadata {
        for (attr_k, attr_v) in meta {
            map.insert(format!("{}/@{}", node.path, attr_k), attr_v.clone());
        }
    }

    if let Some(children) = &node.children {
        for child in children {
            collect_leaf_values(child, map);
        }
    }
}
