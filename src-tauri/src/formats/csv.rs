use super::adapter::FormatAdapter;
use super::capabilities::FormatCapabilities;
use crate::document::{DocumentNode, NodeKind};
use crate::parsers::csv::{check, simple_fix};
use crate::FormatError;

pub struct CsvAdapter;

impl FormatAdapter for CsvAdapter {
    fn id(&self) -> &'static str {
        "csv"
    }

    fn name(&self) -> &'static str {
        "CSV"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &[".csv"]
    }

    fn sniff(&self, content: &str) -> u8 {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return 0;
        }
        let lines: Vec<&str> = trimmed.lines().collect();
        if lines.len() > 1 && lines.iter().all(|l| l.contains(',')) {
            return 80;
        }
        if lines.len() > 1 && trimmed.contains(',') {
            return 65;
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
        let rows = parse_delimited(content, ',');
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
        serialize_delimited(document, ',')
    }

    fn capabilities(&self) -> FormatCapabilities {
        FormatCapabilities {
            supports_repair: true,
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

pub fn parse_delimited(content: &str, delimiter: char) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut current_row = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if in_quotes && chars.peek() == Some(&'"') {
                    current_field.push('"');
                    chars.next();
                } else {
                    in_quotes = !in_quotes;
                }
            }
            ch if ch == delimiter && !in_quotes => {
                current_row.push(current_field);
                current_field = String::new();
            }
            '\n' if !in_quotes => {
                current_row.push(current_field);
                current_field = String::new();
                rows.push(current_row);
                current_row = Vec::new();
            }
            '\r' if !in_quotes => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                current_row.push(current_field);
                current_field = String::new();
                rows.push(current_row);
                current_row = Vec::new();
            }
            _ => {
                current_field.push(c);
            }
        }
    }

    if !current_field.is_empty() || !current_row.is_empty() {
        current_row.push(current_field);
        rows.push(current_row);
    }

    rows
}

pub fn serialize_delimited(
    document: &DocumentNode,
    delimiter: char,
) -> Result<String, FormatError> {
    let mut out = String::new();
    if let Some(rows) = &document.children {
        for (r_idx, row) in rows.iter().enumerate() {
            if r_idx > 0 {
                out.push('\n');
            }
            if let Some(cells) = &row.children {
                for (c_idx, cell) in cells.iter().enumerate() {
                    if c_idx > 0 {
                        out.push(delimiter);
                    }
                    let val = match &cell.value {
                        Some(serde_json::Value::String(s)) => s.as_str(),
                        Some(v) => &v.to_string(),
                        None => "",
                    };
                    if val.contains(delimiter)
                        || val.contains('"')
                        || val.contains('\n')
                        || val.contains('\r')
                    {
                        out.push('"');
                        out.push_str(&val.replace('"', "\"\""));
                        out.push('"');
                    } else {
                        out.push_str(val);
                    }
                }
            }
        }
    }
    Ok(out)
}
