use super::adapter::FormatAdapter;
use super::capabilities::FormatCapabilities;
use super::csv::{parse_delimited, serialize_delimited};
use crate::document::{DocumentNode, NodeKind};
use crate::parsers::json;
use crate::FormatError;

pub fn strip_comments(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut in_string = false;
    let mut escape = false;
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if escape {
            result.push(c);
            escape = false;
            continue;
        }
        if c == '\\' && in_string {
            escape = true;
            result.push(c);
            continue;
        }
        if c == '"' {
            in_string = !in_string;
            result.push(c);
            continue;
        }
        if !in_string && c == '/' {
            if let Some(&'/') = chars.peek() {
                chars.next();
                for next_c in chars.by_ref() {
                    if next_c == '\n' {
                        result.push('\n');
                        break;
                    }
                }
                continue;
            } else if let Some(&'*') = chars.peek() {
                chars.next();
                while let Some(next_c) = chars.next() {
                    if next_c == '*' && chars.peek() == Some(&'/') {
                        chars.next();
                        break;
                    }
                }
                continue;
            }
        }
        result.push(c);
    }
    result
}

// ==================== JSONC ====================
pub struct JsoncAdapter;

impl FormatAdapter for JsoncAdapter {
    fn id(&self) -> &'static str {
        "jsonc"
    }

    fn name(&self) -> &'static str {
        "JSONC"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &[".jsonc"]
    }

    fn sniff(&self, content: &str) -> u8 {
        let trimmed = content.trim();
        if (trimmed.starts_with('{') || trimmed.starts_with('['))
            && (trimmed.contains("//") || trimmed.contains("/*"))
        {
            return 90;
        }
        0
    }

    fn validate(&self, content: &str) -> Vec<FormatError> {
        let stripped = strip_comments(content);
        json::check(&stripped).0
    }

    fn repair(&self, content: &str) -> Option<String> {
        let fixed = json::simple_fix(content);
        if fixed != content {
            Some(fixed)
        } else {
            None
        }
    }

    fn format(&self, content: &str) -> Result<String, FormatError> {
        let stripped = strip_comments(content);
        let val: serde_json::Value = serde_json::from_str(&stripped).map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "JSONC 格式化失败".into(),
        })?;
        serde_json::to_string_pretty(&val).map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "JSONC 序列化失败".into(),
        })
    }

    fn parse(&self, content: &str) -> Result<DocumentNode, FormatError> {
        let stripped = strip_comments(content);
        let val: serde_json::Value = serde_json::from_str(&stripped).map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "无法解析为 JSONC 结构".into(),
        })?;
        let mut node = crate::document::json_value_to_node(&val, None, "/");
        node.refresh_ids_and_paths();
        Ok(node)
    }

    fn serialize(&self, document: &DocumentNode) -> Result<String, FormatError> {
        let val = crate::document::node_to_json_value(document)?;
        serde_json::to_string_pretty(&val).map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "序列化 JSONC 失败".into(),
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

// ==================== JSON5 ====================
pub struct Json5Adapter;

impl FormatAdapter for Json5Adapter {
    fn id(&self) -> &'static str {
        "json5"
    }

    fn name(&self) -> &'static str {
        "JSON5"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &[".json5"]
    }

    fn sniff(&self, content: &str) -> u8 {
        let trimmed = content.trim();
        if (trimmed.starts_with('{') || trimmed.starts_with('['))
            && (trimmed.contains('\'') || trimmed.contains("//") || trimmed.contains(",\n}"))
        {
            return 85;
        }
        0
    }

    fn validate(&self, content: &str) -> Vec<FormatError> {
        let fixed = json::simple_fix(content);
        json::check(&fixed).0
    }

    fn repair(&self, content: &str) -> Option<String> {
        let fixed = json::simple_fix(content);
        if fixed != content {
            Some(fixed)
        } else {
            None
        }
    }

    fn format(&self, content: &str) -> Result<String, FormatError> {
        let fixed = json::simple_fix(content);
        let val: serde_json::Value = serde_json::from_str(&fixed).map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "JSON5 格式化失败".into(),
        })?;
        serde_json::to_string_pretty(&val).map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "JSON5 序列化失败".into(),
        })
    }

    fn parse(&self, content: &str) -> Result<DocumentNode, FormatError> {
        let fixed = json::simple_fix(content);
        let val: serde_json::Value = serde_json::from_str(&fixed).map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "无法解析为 JSON5 结构".into(),
        })?;
        let mut node = crate::document::json_value_to_node(&val, None, "/");
        node.refresh_ids_and_paths();
        Ok(node)
    }

    fn serialize(&self, document: &DocumentNode) -> Result<String, FormatError> {
        let val = crate::document::node_to_json_value(document)?;
        serde_json::to_string_pretty(&val).map_err(|e| FormatError {
            line: None,
            col: None,
            near: None,
            raw: e.to_string(),
            friendly: "序列化 JSON5 失败".into(),
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

// ==================== JSONL ====================
pub struct JsonlAdapter;

impl FormatAdapter for JsonlAdapter {
    fn id(&self) -> &'static str {
        "jsonl"
    }

    fn name(&self) -> &'static str {
        "JSONL"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &[".jsonl", ".ndjson"]
    }

    fn sniff(&self, content: &str) -> u8 {
        let trimmed = content.trim();
        let lines: Vec<&str> = trimmed.lines().filter(|l| !l.trim().is_empty()).collect();
        if lines.len() > 1
            && lines
                .iter()
                .all(|l| l.trim().starts_with('{') && l.trim().ends_with('}'))
        {
            return 88;
        }
        0
    }

    fn validate(&self, content: &str) -> Vec<FormatError> {
        let mut errors = Vec::new();
        for (i, line) in content.lines().enumerate() {
            let t = line.trim();
            if t.is_empty() {
                continue;
            }
            if let Err(e) = serde_json::from_str::<serde_json::Value>(t) {
                errors.push(FormatError {
                    line: Some((i + 1) as u32),
                    col: None,
                    near: Some(t.chars().take(40).collect()),
                    raw: e.to_string(),
                    friendly: format!("第 {} 行 JSONL 存在格式错误", i + 1),
                });
            }
        }
        errors
    }

    fn repair(&self, content: &str) -> Option<String> {
        let mut repaired = Vec::new();
        let mut modified = false;
        for line in content.lines() {
            let t = line.trim();
            if t.is_empty() {
                continue;
            }
            let fixed = json::simple_fix(t);
            if fixed != t {
                modified = true;
            }
            repaired.push(fixed);
        }
        if modified {
            Some(repaired.join("\n"))
        } else {
            None
        }
    }

    fn format(&self, content: &str) -> Result<String, FormatError> {
        let mut lines = Vec::new();
        for (i, line) in content.lines().enumerate() {
            let t = line.trim();
            if t.is_empty() {
                continue;
            }
            let val: serde_json::Value = serde_json::from_str(t).map_err(|e| FormatError {
                line: Some((i + 1) as u32),
                col: None,
                near: None,
                raw: e.to_string(),
                friendly: format!("第 {} 行格式化失败", i + 1),
            })?;
            let minified = serde_json::to_string(&val).map_err(|e| FormatError {
                line: None,
                col: None,
                near: None,
                raw: e.to_string(),
                friendly: "JSONL 压缩失败".into(),
            })?;
            lines.push(minified);
        }
        Ok(lines.join("\n"))
    }

    fn parse(&self, content: &str) -> Result<DocumentNode, FormatError> {
        let mut root = DocumentNode::new(NodeKind::Array, "/");
        let mut children = Vec::new();

        for (i, line) in content.lines().enumerate() {
            let t = line.trim();
            if t.is_empty() {
                continue;
            }
            let val: serde_json::Value = serde_json::from_str(t).map_err(|e| FormatError {
                line: Some((i + 1) as u32),
                col: None,
                near: None,
                raw: e.to_string(),
                friendly: format!("第 {} 行无法解析为 JSON", i + 1),
            })?;
            let child = crate::document::json_value_to_node(&val, None, &format!("/{}", i));
            children.push(child);
        }

        root.children = Some(children);
        root.refresh_ids_and_paths();
        Ok(root)
    }

    fn serialize(&self, document: &DocumentNode) -> Result<String, FormatError> {
        let mut lines = Vec::new();
        if let Some(children) = &document.children {
            for child in children {
                let val = crate::document::node_to_json_value(child)?;
                let s = serde_json::to_string(&val).map_err(|e| FormatError {
                    line: None,
                    col: None,
                    near: None,
                    raw: e.to_string(),
                    friendly: "JSONL 序列化失败".into(),
                })?;
                lines.push(s);
            }
        }
        Ok(lines.join("\n"))
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

// ==================== TSV ====================
pub struct TsvAdapter;

impl FormatAdapter for TsvAdapter {
    fn id(&self) -> &'static str {
        "tsv"
    }

    fn name(&self) -> &'static str {
        "TSV"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &[".tsv", ".tab"]
    }

    fn sniff(&self, content: &str) -> u8 {
        let trimmed = content.trim();
        let lines: Vec<&str> = trimmed.lines().collect();
        if lines.len() > 1 && lines.iter().all(|l| l.contains('\t')) {
            return 85;
        }
        0
    }

    fn validate(&self, content: &str) -> Vec<FormatError> {
        let rows = parse_delimited(content, '\t');
        let mut errors = Vec::new();
        if rows.is_empty() {
            return errors;
        }
        let expected_cols = rows[0].len();
        for (i, row) in rows.iter().enumerate() {
            if row.len() != expected_cols {
                errors.push(FormatError {
                    line: Some((i + 1) as u32),
                    col: None,
                    near: None,
                    raw: format!(
                        "列数不一致：首行 {} 列，当前 {} 列",
                        expected_cols,
                        row.len()
                    ),
                    friendly: format!("第 {} 行列数与表头不一致", i + 1),
                });
            }
        }
        errors
    }

    fn repair(&self, _content: &str) -> Option<String> {
        None
    }

    fn format(&self, content: &str) -> Result<String, FormatError> {
        let doc = self.parse(content)?;
        self.serialize(&doc)
    }

    fn parse(&self, content: &str) -> Result<DocumentNode, FormatError> {
        let rows = parse_delimited(content, '\t');
        let mut table = DocumentNode::new(NodeKind::Table, "/");
        let mut row_nodes = Vec::with_capacity(rows.len());

        for (r_idx, row) in rows.iter().enumerate() {
            let mut row_node = DocumentNode::new(NodeKind::Row, format!("/{}", r_idx));
            let mut cell_nodes = Vec::with_capacity(row.len());
            for (c_idx, cell) in row.iter().enumerate() {
                let mut cell_node =
                    DocumentNode::new(NodeKind::String, format!("/{}/{}", r_idx, c_idx));
                cell_node.value = Some(serde_json::Value::String(cell.clone()));
                cell_nodes.push(cell_node);
            }
            row_node.children = Some(cell_nodes);
            row_nodes.push(row_node);
        }

        table.children = Some(row_nodes);
        table.refresh_ids_and_paths();
        Ok(table)
    }

    fn serialize(&self, document: &DocumentNode) -> Result<String, FormatError> {
        serialize_delimited(document, '\t')
    }

    fn capabilities(&self) -> FormatCapabilities {
        FormatCapabilities {
            supports_repair: false,
            supports_format: true,
            supports_tree_editor: false,
            supports_grid_editor: true,
            supports_kv_editor: false,
            supports_dom_editor: false,
            supports_schema: false,
            preserve_comments: false,
        }
    }
}

// ==================== PROPERTIES ====================
pub struct PropertiesAdapter;

impl FormatAdapter for PropertiesAdapter {
    fn id(&self) -> &'static str {
        "properties"
    }

    fn name(&self) -> &'static str {
        "Properties"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &[".properties"]
    }

    fn sniff(&self, content: &str) -> u8 {
        let trimmed = content.trim();
        let lines: Vec<&str> = trimmed.lines().collect();
        let has_prop_comment = lines
            .iter()
            .any(|l| l.trim().starts_with('#') || l.trim().starts_with('!'));
        let has_colon_assign = lines
            .iter()
            .any(|l| l.contains(':') && !l.trim().starts_with('#'));
        if has_prop_comment && has_colon_assign {
            return 80;
        }
        0
    }

    fn validate(&self, _content: &str) -> Vec<FormatError> {
        Vec::new()
    }

    fn repair(&self, _content: &str) -> Option<String> {
        None
    }

    fn format(&self, content: &str) -> Result<String, FormatError> {
        let doc = self.parse(content)?;
        self.serialize(&doc)
    }

    fn parse(&self, content: &str) -> Result<DocumentNode, FormatError> {
        let mut root = DocumentNode::new(NodeKind::Object, "/");
        let mut kvs = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
                continue;
            }
            let sep_pos = trimmed.find('=').or_else(|| trimmed.find(':'));
            if let Some(pos) = sep_pos {
                let key = trimmed[..pos].trim();
                let val = trimmed[pos + 1..].trim();
                let mut node = DocumentNode::new(NodeKind::KeyValue, format!("/{}", key));
                node.key = Some(key.to_string());
                node.value = Some(serde_json::Value::String(val.to_string()));
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
                    let v = match &child.value {
                        Some(serde_json::Value::String(s)) => s.clone(),
                        Some(val) => val.to_string(),
                        None => String::new(),
                    };
                    out.push_str(k);
                    out.push_str(" = ");
                    out.push_str(&v);
                    out.push('\n');
                }
            }
        }
        Ok(out)
    }

    fn capabilities(&self) -> FormatCapabilities {
        FormatCapabilities {
            supports_repair: false,
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

// ==================== EditorConfig ====================
pub struct EditorConfigAdapter;

impl FormatAdapter for EditorConfigAdapter {
    fn id(&self) -> &'static str {
        "editorconfig"
    }

    fn name(&self) -> &'static str {
        "EditorConfig"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &[".editorconfig"]
    }

    fn sniff(&self, content: &str) -> u8 {
        let trimmed = content.trim();
        if trimmed.starts_with("root = true")
            || trimmed.contains("[*]")
            || trimmed.contains("indent_style")
        {
            return 95;
        }
        0
    }

    fn validate(&self, _content: &str) -> Vec<FormatError> {
        Vec::new()
    }

    fn repair(&self, _content: &str) -> Option<String> {
        None
    }

    fn format(&self, content: &str) -> Result<String, FormatError> {
        let doc = self.parse(content)?;
        self.serialize(&doc)
    }

    fn parse(&self, content: &str) -> Result<DocumentNode, FormatError> {
        crate::formats::ini::IniAdapter.parse(content)
    }

    fn serialize(&self, document: &DocumentNode) -> Result<String, FormatError> {
        crate::formats::ini::IniAdapter.serialize(document)
    }

    fn capabilities(&self) -> FormatCapabilities {
        FormatCapabilities {
            supports_repair: false,
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

// ==================== GitConfig ====================
pub struct GitConfigAdapter;

impl FormatAdapter for GitConfigAdapter {
    fn id(&self) -> &'static str {
        "gitconfig"
    }

    fn name(&self) -> &'static str {
        "GitConfig"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &[".gitconfig"]
    }

    fn sniff(&self, content: &str) -> u8 {
        let trimmed = content.trim();
        if trimmed.contains("[user]") || trimmed.contains("[core]") || trimmed.contains("[remote ")
        {
            return 95;
        }
        0
    }

    fn validate(&self, _content: &str) -> Vec<FormatError> {
        Vec::new()
    }

    fn repair(&self, _content: &str) -> Option<String> {
        None
    }

    fn format(&self, content: &str) -> Result<String, FormatError> {
        let doc = self.parse(content)?;
        self.serialize(&doc)
    }

    fn parse(&self, content: &str) -> Result<DocumentNode, FormatError> {
        crate::formats::ini::IniAdapter.parse(content)
    }

    fn serialize(&self, document: &DocumentNode) -> Result<String, FormatError> {
        crate::formats::ini::IniAdapter.serialize(document)
    }

    fn capabilities(&self) -> FormatCapabilities {
        FormatCapabilities {
            supports_repair: false,
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

// ==================== HCL / Terraform ====================
pub struct HclAdapter;

impl FormatAdapter for HclAdapter {
    fn id(&self) -> &'static str {
        "hcl"
    }

    fn name(&self) -> &'static str {
        "HCL / Terraform"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &[".hcl", ".tf", ".tfvars"]
    }

    fn sniff(&self, content: &str) -> u8 {
        let trimmed = content.trim();
        if (trimmed.contains("variable ")
            || trimmed.contains("resource ")
            || trimmed.contains("terraform "))
            && trimmed.contains('{')
        {
            return 90;
        }
        0
    }

    fn validate(&self, _content: &str) -> Vec<FormatError> {
        Vec::new()
    }

    fn repair(&self, _content: &str) -> Option<String> {
        None
    }

    fn format(&self, content: &str) -> Result<String, FormatError> {
        let doc = self.parse(content)?;
        self.serialize(&doc)
    }

    fn parse(&self, content: &str) -> Result<DocumentNode, FormatError> {
        let mut root = DocumentNode::new(NodeKind::Object, "/");
        let mut children = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
                continue;
            }
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                let mut val = trimmed[eq_pos + 1..].trim();
                if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
                    val = &val[1..val.len() - 1];
                }
                let mut node = DocumentNode::new(NodeKind::KeyValue, format!("/{}", key));
                node.key = Some(key.to_string());
                node.value = Some(serde_json::Value::String(val.to_string()));
                children.push(node);
            }
        }

        root.children = Some(children);
        root.refresh_ids_and_paths();
        Ok(root)
    }

    fn serialize(&self, document: &DocumentNode) -> Result<String, FormatError> {
        let mut out = String::new();
        if let Some(children) = &document.children {
            for child in children {
                if let Some(k) = &child.key {
                    let v = match &child.value {
                        Some(serde_json::Value::String(s)) => format!("\"{}\"", s),
                        Some(val) => val.to_string(),
                        None => "\"\"".into(),
                    };
                    out.push_str(k);
                    out.push_str(" = ");
                    out.push_str(&v);
                    out.push('\n');
                }
            }
        }
        Ok(out)
    }

    fn capabilities(&self) -> FormatCapabilities {
        FormatCapabilities {
            supports_repair: false,
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
