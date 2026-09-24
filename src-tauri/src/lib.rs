pub mod document;
pub mod formats;
mod parsers;
pub mod profiles;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatError {
    pub line: Option<u32>,
    pub col: Option<u32>,
    pub near: Option<String>,
    pub raw: String,
    pub friendly: String,
}

#[cfg(test)]
mod sample_fix_tests {
    /// 辅助函数：修复后验证内容合法
    fn assert_fixes_to_valid(format: &str, content: &str) {
        let fixed = super::simple_fix(content, format);
        let result = super::check_format(&fixed, format);
        if !result.valid {
            panic!(
                "[{}] 修复后仍然无效:\n输入:\n{}\n修复后:\n{}\n错误: {}",
                format,
                content,
                fixed,
                result
                    .errors
                    .iter()
                    .map(|e| e.raw.as_str())
                    .collect::<Vec<_>>()
                    .join(" | ")
            );
        }
    }

    // ==================== JSON 测试 ====================

    #[test]
    fn json_basic_sample() {
        assert_fixes_to_valid(
            "json",
            r#"{
  // comment
  title: 'Orthos',
  "version": undefined,
  "items": [
    { "name": "alpha", "count": 1 },
    { "name": "beta", "count": 2, },
  ],
}"#,
        );
    }

    #[test]
    fn json_trailing_comma_in_array() {
        assert_fixes_to_valid("json", r#"{"items": [1, 2, 3,]}"#);
    }

    #[test]
    fn json_trailing_comma_in_object() {
        assert_fixes_to_valid("json", r#"{"a": 1, "b": 2,}"#);
    }

    #[test]
    fn json_trailing_comma_nested() {
        assert_fixes_to_valid("json", r#"{"a": {"b": [1, 2,],},}"#);
    }

    #[test]
    fn json_single_quotes() {
        assert_fixes_to_valid("json", r#"{'name': 'test', 'value': 42}"#);
    }

    #[test]
    fn json_unquoted_keys() {
        assert_fixes_to_valid("json", r#"{name: "test", value: 42}"#);
    }

    #[test]
    fn json_single_quotes_with_inner_double_quote() {
        assert_fixes_to_valid("json", r#"{'key': 'value with "quotes" inside'}"#);
    }

    #[test]
    fn json_line_comment() {
        assert_fixes_to_valid("json", "{\n  \"a\": 1, // line comment\n  \"b\": 2\n}");
    }

    #[test]
    fn json_block_comment() {
        assert_fixes_to_valid("json", "{\n  \"a\": 1, /* block comment */\n  \"b\": 2\n}");
    }

    #[test]
    fn json_undefined_value() {
        assert_fixes_to_valid("json", r#"{"a": undefined, "b": "undefined"}"#);
    }

    #[test]
    fn json_missing_closing_brace() {
        assert_fixes_to_valid("json", r#"{"a": 1, "b": [1, 2]"#);
    }

    #[test]
    fn json_missing_closing_bracket() {
        assert_fixes_to_valid("json", r#"{"items": [1, 2, 3}"#);
    }

    #[test]
    fn json_missing_comma_between_values() {
        assert_fixes_to_valid("json", "{\n  \"a\": 1\n  \"b\": 2\n}");
    }

    #[test]
    fn json_missing_comma_after_closing_brace() {
        assert_fixes_to_valid("json", "{\n  \"a\": {\"x\": 1}\n  \"b\": 2\n}");
    }

    #[test]
    fn json_mixed_single_quote_and_trailing_comma() {
        assert_fixes_to_valid("json", "{\n  'name': 'test',\n  'items': [1, 2,],\n}");
    }

    #[test]
    fn json_deeply_nested_missing_brackets() {
        assert_fixes_to_valid(
            "json",
            "{\n  \"a\": {\n    \"b\": {\n      \"c\": 1\n    }\n  }\n",
        );
    }

    #[test]
    fn json_unicode_content() {
        assert_fixes_to_valid("json", r#"{"名称": "测试", "值": 42}"#);
    }

    #[test]
    fn json_comment_inside_string_not_removed() {
        assert_fixes_to_valid("json", r#"{"url": "http://example.com/path"}"#);
    }

    #[test]
    fn json_block_comment_multiline() {
        assert_fixes_to_valid(
            "json",
            "{\n  \"a\": 1,\n  /* multi\n     line\n     comment */\n  \"b\": 2\n}",
        );
    }

    #[test]
    fn json_empty_object_and_array() {
        assert_fixes_to_valid("json", r#"{"a": [], "b": {}}"#);
    }

    #[test]
    fn json_values_types() {
        assert_fixes_to_valid(
            "json",
            r#"{"str": "hello", "num": 42, "float": 3.14, "bool": true, "null": null}"#,
        );
    }

    // ==================== YAML 测试 ====================

    #[test]
    fn yaml_basic_sample() {
        assert_fixes_to_valid(
            "yaml",
            "app:Orthos\npaths:\n\t- ./config.json\n\t- ./config.yaml\nusers:\n  admin: true\n  admin_enabled: false\nempty_item:\n  -\nflow_list: [1,2,3]\nflow_map: {host:localhost,port:8080}\n",
        );
    }

    #[test]
    fn yaml_tab_indentation() {
        assert_fixes_to_valid("yaml", "name: test\nitems:\n\t- one\n\t- two");
    }

    #[test]
    fn yaml_colon_no_space() {
        assert_fixes_to_valid("yaml", "name:test\nversion:1.0");
    }

    #[test]
    fn yaml_duplicate_keys_are_preserved() {
        let input = "name: first\nname: second";
        let fixed = super::simple_fix(input, "yaml");
        assert_eq!(fixed, input);
        let result = super::check_format(&fixed, "yaml");
        assert!(!result.valid);
        assert!(result.corrected.is_none());
    }

    #[test]
    fn yaml_duplicate_keys_nested_are_preserved() {
        let input = "server:\n  host: localhost\n  host: 0.0.0.0\n  port: 8080";
        let fixed = super::simple_fix(input, "yaml");
        assert_eq!(fixed, input);
        let result = super::check_format(&fixed, "yaml");
        assert!(!result.valid);
        assert!(result.corrected.is_none());
    }

    #[test]
    fn yaml_flow_sequence_spacing() {
        assert_fixes_to_valid("yaml", "list: [1,2,3,4,5]");
    }

    #[test]
    fn yaml_flow_mapping_spacing() {
        assert_fixes_to_valid("yaml", "map: {host:localhost,port:8080}");
    }

    #[test]
    fn yaml_empty_list_item() {
        assert_fixes_to_valid("yaml", "items:\n  -\n  - name: test");
    }

    #[test]
    fn yaml_dash_no_space() {
        assert_fixes_to_valid("yaml", "items:\n  -name: test\n  -value: 1");
    }

    #[test]
    fn yaml_mixed_issues() {
        assert_fixes_to_valid(
            "yaml",
            "app:MyApp\nversion:1.0\ntabs:\n\t- a\n\t- b\ndup: first\ndup2: second\nflow: [x,y,z]",
        );
    }

    #[test]
    fn user_broken_json_sample_fixes_to_valid() {
        assert_fixes_to_valid(
            "json",
            r#"{
  // comment: JSON 标准不允许注释
  title: 'Orthos',
  "version": undefined,
  "enabled": true,
  "items": [
    { "name": "alpha", "count": 1 },
    { "name": "beta", "count": 2, },
  ],
}"#,
        );
    }

    #[test]
    fn user_broken_toml_sample_fixes_to_valid() {
        assert_fixes_to_valid(
            "toml",
            "[server\nhost=\"localhost\"\nport 8080\ndebug = true\ndebug_enabled = false\nbad key = \"needs quotes\"\nmessage = \"hello",
        );
    }

    #[test]
    fn user_broken_yaml_sample_fixes_to_valid() {
        assert_fixes_to_valid(
            "yaml",
            "app:Orthos\npaths:\n\t- ./config.json\n\t- ./config.yaml\nusers:\n  admin: true\n  admin_enabled: false\nempty_item:\n  -\nflow_list: [1,2,3]\nflow_map: {host:localhost,port:8080}",
        );
    }

    #[test]
    fn yaml_complex_nested() {
        assert_fixes_to_valid(
            "yaml",
            "database:\n  host:localhost\n  port:5432\n  credentials:\n    user:admin\n    pass:secret",
        );
    }

    #[test]
    fn yaml_list_with_mapping_items() {
        // 简化测试：列表项键名无引号
        assert_fixes_to_valid(
            "yaml",
            "users:\n  -name: Alice\n  -name: Bob\n  -name: Charlie",
        );
    }

    #[test]
    fn yaml_trailing_whitespace() {
        assert_fixes_to_valid("yaml", "name: test   \nversion: 1.0  ");
    }

    // ==================== TOML 测试 ====================

    #[test]
    fn toml_basic_sample() {
        assert_fixes_to_valid(
            "toml",
            "[server\nhost=\"localhost\"\nport 8080\ndebug = true\ndebug_enabled = false\nbad key = \"needs quotes\"\nmessage = \"hello\n",
        );
    }

    #[test]
    fn toml_unclosed_section() {
        assert_fixes_to_valid("toml", "[server\nhost = \"localhost\"");
    }

    #[test]
    fn toml_unclosed_section_with_content() {
        assert_fixes_to_valid(
            "toml",
            "[server]\nhost = \"localhost\"\n[database\nname = \"test\"",
        );
    }

    #[test]
    fn toml_duplicate_keys_are_preserved() {
        let input = "name = \"first\"\nname = \"second\"";
        let fixed = super::simple_fix(input, "toml");
        assert_eq!(fixed, input);
        let result = super::check_format(&fixed, "toml");
        assert!(!result.valid);
        assert!(result.corrected.is_none());
    }

    #[test]
    fn toml_duplicate_keys_in_section_are_preserved() {
        let input = "[server]\nhost = \"a\"\nhost = \"b\"\nport = 8080";
        let fixed = super::simple_fix(input, "toml");
        assert_eq!(fixed, input);
        let result = super::check_format(&fixed, "toml");
        assert!(!result.valid);
        assert!(result.corrected.is_none());
    }

    #[test]
    fn toml_missing_equals() {
        assert_fixes_to_valid("toml", "[server]\nhost \"localhost\"\nport 8080");
    }

    #[test]
    fn toml_unclosed_quote() {
        assert_fixes_to_valid("toml", "[server]\nmessage = \"hello");
    }

    #[test]
    fn toml_unclosed_single_quote() {
        assert_fixes_to_valid("toml", "[server]\nmessage = 'hello");
    }

    #[test]
    fn toml_triple_quote_unclosed() {
        assert_fixes_to_valid("toml", "[server]\nmessage = \"\"\"hello world");
    }

    #[test]
    fn toml_single_quote_triple_unclosed() {
        assert_fixes_to_valid("toml", "[server]\nmessage = '''hello world");
    }

    #[test]
    fn toml_array_of_tables_unclosed() {
        assert_fixes_to_valid(
            "toml",
            "[[fruit]]\nname = \"apple\"\n\n[[fruit]]\nname = \"banana\"",
        );
    }

    #[test]
    fn toml_special_chars_in_key() {
        assert_fixes_to_valid("toml", "[server]\nbad key = \"value\"");
    }

    #[test]
    fn toml_mixed_issues() {
        assert_fixes_to_valid(
            "toml",
            "[server\nhost=\"localhost\"\nport 8080\ndebug = true\ndebug_enabled = false",
        );
    }

    #[test]
    fn toml_key_no_quotes_with_space() {
        assert_fixes_to_valid("toml", "[server]\nmy key = \"value\"");
    }

    // ==================== XML 测试 ====================

    #[test]
    fn xml_basic_sample() {
        assert_fixes_to_valid(
            "xml",
            "<config>\n  <app name=Orthos>\n    <title>Orthos</titel>\n    <feature>yaml\n  </app>\n</config>\n",
        );
    }

    #[test]
    fn xml_unclosed_tag() {
        assert_fixes_to_valid("xml", "<root><item>text</root>");
    }

    #[test]
    fn xml_mismatched_tag() {
        assert_fixes_to_valid("xml", "<root><item>text</other></root>");
    }

    #[test]
    fn xml_unquoted_attribute() {
        assert_fixes_to_valid("xml", r#"<root name=test value="ok">"#);
    }

    #[test]
    fn xml_unclosed_comment() {
        assert_fixes_to_valid("xml", "<root><!-- comment</root>");
    }

    #[test]
    fn xml_unclosed_cdata() {
        assert_fixes_to_valid("xml", "<root><![CDATA[some data</root>");
    }

    #[test]
    fn xml_self_closing_tag() {
        assert_fixes_to_valid("xml", "<root><br/><hr><img src=\"test.png\"/></root>");
    }

    #[test]
    fn xml_void_named_element_uses_generic_xml_rules() {
        let fixed = super::simple_fix("<root><br></root>", "xml");
        assert_eq!(fixed, "<root><br></br></root>");
        assert_fixes_to_valid("xml", &fixed);
    }

    #[test]
    fn xml_nested_unclosed() {
        assert_fixes_to_valid("xml", "<root><a><b>text</a></root>");
    }

    #[test]
    fn xml_comment_with_greater_than() {
        assert_fixes_to_valid("xml", "<root><!-- a > b --><item/></root>");
    }

    #[test]
    fn xml_cdata_with_greater_than() {
        assert_fixes_to_valid("xml", "<root><![CDATA[data > 0]]></root>");
    }

    #[test]
    fn xml_processing_instruction() {
        assert_fixes_to_valid("xml", "<root><?xml version=\"1.0\"?><item/></root>");
    }

    // ==================== CSV 测试 ====================

    #[test]
    fn csv_basic_sample() {
        assert_fixes_to_valid(
            "csv",
            "name,age,email\nAlice,30,alice@example.com\nBob,25,bob@example.com\nCharlie,40,\n中文用户,28,zh@example.com\n",
        );
    }

    #[test]
    fn csv_too_few_columns() {
        assert_fixes_to_valid("csv", "name,age,email\nAlice,30");
    }

    #[test]
    fn csv_too_many_columns_are_preserved() {
        let input = "name,age\nAlice,30,extra";
        let fixed = super::simple_fix(input, "csv");

        assert_eq!(fixed, input);
        let result = super::check_format(&fixed, "csv");
        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn csv_bom() {
        assert_fixes_to_valid("csv", "\u{feff}name,age\nAlice,30");
    }

    #[test]
    fn csv_crlf() {
        assert_fixes_to_valid("csv", "name,age\r\nAlice,30\r\nBob,25");
    }

    #[test]
    fn csv_unclosed_quote() {
        assert_fixes_to_valid("csv", "name,age\n\"Alice,30");
    }

    #[test]
    fn csv_escaped_quotes() {
        assert_fixes_to_valid("csv", "name,description\nAlice,\"she said \"\"hello\"\"\"");
    }

    #[test]
    fn csv_empty_lines() {
        assert_fixes_to_valid("csv", "name,age\n\nAlice,30\n\nBob,25");
    }

    #[test]
    fn csv_tab_delimiter() {
        assert_fixes_to_valid("csv", "name\tage\nAlice\t30\nBob\t25");
    }

    #[test]
    fn csv_semicolon_delimiter() {
        assert_fixes_to_valid("csv", "name;age\nAlice;30\nBob;25");
    }

    #[test]
    fn csv_pipe_delimiter() {
        assert_fixes_to_valid("csv", "name|age\nAlice|30\nBob|25");
    }

    #[test]
    fn csv_trailing_extra_field_is_preserved() {
        let input = "name,age,email\nAlice,30,alice@test.com,";
        let fixed = super::simple_fix(input, "csv");

        assert_eq!(fixed, input);
        let result = super::check_format(&fixed, "csv");
        assert!(!result.valid);
    }

    #[test]
    fn csv_quote_with_comma_inside() {
        assert_fixes_to_valid("csv", "name,desc\nAlice,\"hello, world\"");
    }

    // ==================== INI 测试 ====================

    #[test]
    fn ini_basic_sample() {
        assert_fixes_to_valid(
            "ini",
            "app_name = Orthos\n[server\nhost=localhost\nport 8080\nhost_backup=127.0.0.1\n[]\n# comment style to normalize\n",
        );
    }

    #[test]
    fn ini_orphan_keys() {
        assert_fixes_to_valid("ini", "name = test\nversion = 1.0");
    }

    #[test]
    fn ini_duplicate_keys_are_preserved() {
        let input = "[server]\nhost = a\nhost = b\nport = 8080";
        let fixed = super::simple_fix(input, "ini");
        assert_eq!(fixed, input);
        let result = super::check_format(&fixed, "ini");
        assert!(!result.valid);
        assert!(result.corrected.is_none());
    }

    #[test]
    fn ini_unclosed_section() {
        assert_fixes_to_valid("ini", "[server\nhost = localhost");
    }

    #[test]
    fn ini_empty_section_name() {
        assert_fixes_to_valid("ini", "[]\nhost = localhost");
    }

    #[test]
    fn ini_comment_style_hash() {
        assert_fixes_to_valid(
            "ini",
            "# this is a comment\n[server]\n# another comment\nhost = localhost",
        );
    }

    #[test]
    fn ini_comment_style_semicolon() {
        assert_fixes_to_valid(
            "ini",
            "; this is a comment\n[server]\n; another comment\nhost = localhost",
        );
    }

    #[test]
    fn ini_missing_equals() {
        assert_fixes_to_valid("ini", "[server]\nhost localhost\nport 8080");
    }

    #[test]
    fn ini_section_header_with_content() {
        assert_fixes_to_valid("ini", "[server] host = localhost\nport = 8080");
    }

    // ==================== ENV 测试 ====================

    #[test]
    fn env_basic_sample() {
        assert_fixes_to_valid(
            "env",
            "APP_NAME=Orthos\n=missing_key\nPORT 8080\nBAD.KEY=value\nQUOTED=\"missing end\n; semicolon comment\nUNICODE_VALUE=中文内容😀\n",
        );
    }

    #[test]
    fn env_missing_equals() {
        assert_fixes_to_valid("env", "NAME\nVALUE=test");
    }

    #[test]
    fn env_unclosed_double_quote() {
        assert_fixes_to_valid("env", "GREETING=\"hello world");
    }

    #[test]
    fn env_unclosed_single_quote() {
        assert_fixes_to_valid("env", "GREETING='hello world");
    }

    #[test]
    fn env_invalid_key_chars() {
        assert_fixes_to_valid("env", "BAD.KEY=value\nBAD-KEY=value2");
    }

    #[test]
    fn env_semicolon_comment() {
        assert_fixes_to_valid("env", "; comment\nNAME=test");
    }

    #[test]
    fn env_bom() {
        assert_fixes_to_valid("env", "\u{feff}NAME=test");
    }

    #[test]
    fn env_crlf() {
        assert_fixes_to_valid("env", "NAME=test\r\nVALUE=123");
    }

    #[test]
    fn env_empty_value() {
        assert_fixes_to_valid("env", "EMPTY=\nNAME=test");
    }

    #[test]
    fn env_value_with_equals() {
        assert_fixes_to_valid("env", "DATABASE_URL=postgres://user:pass@host:5432/db");
    }

    // ==================== 通用测试 ====================

    #[test]
    fn all_formats_fix_to_valid() {
        let samples = [
            (
                "json",
                r#"{
  // comment
  title: 'Orthos',
  "version": undefined,
  "items": [
    { "name": "alpha", "count": 1 },
    { "name": "beta", "count": 2, },
  ],
}"#,
            ),
            (
                "yaml",
                "app:Orthos\npaths:\n\t- ./config.json\n\t- ./config.yaml\nusers:\n  admin: true\n  admin_enabled: false\nempty_item:\n  -\nflow_list: [1,2,3]\nflow_map: {host:localhost,port:8080}\n",
            ),
            (
                "toml",
                "[server\nhost=\"localhost\"\nport 8080\ndebug = true\ndebug_enabled = false\nbad key = \"needs quotes\"\nmessage = \"hello\n",
            ),
            (
                "xml",
                "<config>\n  <app name=Orthos>\n    <title>Orthos</titel>\n    <feature>yaml\n  </app>\n</config>\n",
            ),
            (
                "csv",
                "name,age,email\nAlice,30,alice@example.com\nBob,25,bob@example.com\nCharlie,40,\n中文用户,28,zh@example.com\n",
            ),
            (
                "ini",
                "app_name = Orthos\n[server\nhost=localhost\nport 8080\nhost_backup=127.0.0.1\n[]\n# comment style to normalize\n",
            ),
            (
                "env",
                "APP_NAME=Orthos\n=missing_key\nPORT 8080\nBAD.KEY=value\nQUOTED=\"missing end\n; semicolon comment\nUNICODE_VALUE=中文内容😀\n",
            ),
        ];

        let mut failures = Vec::new();
        for (format, content) in samples {
            let fixed = super::simple_fix(content, format);
            let result = super::check_format(&fixed, format);
            if !result.valid {
                failures.push(format!(
                    "{} still invalid after fix: {}\nfixed:\n{}",
                    format,
                    result
                        .errors
                        .iter()
                        .map(|e| e.raw.as_str())
                        .collect::<Vec<_>>()
                        .join(" | "),
                    fixed
                ));
            }
        }

        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    // ==================== 调试测试 ====================

    #[test]
    fn debug_xml_cdata() {
        let input = "<root><![CDATA[data > 0]]></root>";
        let result = super::parsers::xml::simple_fix(input);
        eprintln!("CDATA input:  {:?}", input);
        eprintln!("CDATA output: {:?}", result);
    }

    #[test]
    fn debug_xml_self_closing() {
        let input = r#"<root><br/><hr><img src="test.png"/></root>"#;
        let result = super::parsers::xml::simple_fix(input);
        eprintln!("SelfClose input:  {:?}", input);
        eprintln!("SelfClose output: {:?}", result);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckResult {
    pub format: String,
    pub valid: bool,
    pub errors: Vec<FormatError>,
    pub corrected: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchItem {
    pub filename: String,
    pub result: CheckResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaCheckRequest {
    pub content: String,
    pub schema: String,
    pub format: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaCheckResult {
    pub valid: bool,
    pub errors: Vec<FormatError>,
}

/// 文件大小限制：10 MB
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

/// 允许读取和保存文件的目录白名单
fn get_allowed_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(desktop) = dirs::desktop_dir() {
        dirs.push(desktop);
    }
    if let Some(document) = dirs::document_dir() {
        dirs.push(document);
    }
    if let Some(download) = dirs::download_dir() {
        dirs.push(download);
    }
    dirs
}

/// 检查目标路径是否在允许的目录内
fn is_path_allowed(target: &Path) -> bool {
    if target
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return false;
    }

    let allowed = get_allowed_dirs();
    let canonical = match target.canonicalize() {
        Ok(p) => p,
        Err(_) => match target.parent().and_then(nearest_existing_ancestor) {
            Some(p) => p,
            None => return false,
        },
    };
    allowed.iter().any(|dir| {
        dir.canonicalize()
            .map(|d| canonical.starts_with(&d))
            .unwrap_or(false)
    })
}

fn nearest_existing_ancestor(path: &Path) -> Option<PathBuf> {
    let mut current = Some(path);
    while let Some(candidate) = current {
        if candidate.exists() {
            return candidate.canonicalize().ok();
        }
        current = candidate.parent();
    }
    None
}

/// Detect format from file extension
pub fn detect_format(filename: &str) -> Option<String> {
    formats::get_registry().detect(Some(filename), None)
}

/// Heuristic format detection from content
pub fn detect_format_heuristic(content: &str) -> Option<String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with('{') {
        Some("json".into())
    } else if trimmed.starts_with('[') {
        detect_bracketed_format(trimmed)
    } else if trimmed.starts_with('<') {
        Some("xml".into())
    } else if trimmed.starts_with("---") || trimmed.contains("\n---\n") {
        Some("yaml".into())
    } else if trimmed.lines().any(|l| {
        let t = l.trim();
        t.starts_with('[') && t.contains(']') && !t.starts_with("[{")
    }) {
        if looks_like_ini(trimmed) {
            Some("ini".into())
        } else if trimmed.lines().any(|l| l.contains('=')) {
            Some("toml".into())
        } else {
            Some("ini".into())
        }
    } else if trimmed.starts_with('#') && trimmed.contains('=') {
        Some("env".into())
    } else if trimmed.lines().any(|l| {
        let t = l.trim();
        t.contains(": ") && !t.starts_with('#')
    }) {
        Some("yaml".into())
    } else if trimmed.contains(',') && trimmed.lines().count() > 1 {
        Some("csv".into())
    } else if trimmed.contains('=') {
        Some("env".into())
    } else {
        None
    }
}

fn detect_bracketed_format(trimmed: &str) -> Option<String> {
    if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
        return Some("json".into());
    }

    let has_section = trimmed.lines().any(|line| {
        let t = line.trim();
        t.starts_with('[') && t.contains(']') && !t.starts_with("[{")
    });

    if has_section {
        if looks_like_ini(trimmed) {
            Some("ini".into())
        } else if trimmed.lines().any(|line| line.contains('=')) {
            Some("toml".into())
        } else {
            Some("ini".into())
        }
    } else {
        Some("json".into())
    }
}

fn looks_like_ini(content: &str) -> bool {
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

fn is_ini_bare_value(value: &str) -> bool {
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

fn run_parser(content: &str, format: &str) -> (Vec<FormatError>, Option<String>) {
    if let Some(adapter) = formats::get_registry().get(format) {
        let errors = adapter.validate(content);
        let corrected = adapter.repair(content);
        (errors, corrected)
    } else {
        (
            vec![FormatError {
                line: None,
                col: None,
                near: None,
                raw: format!("不支持的格式: {}", format),
                friendly: format!("暂不支持 {} 格式的校验", format),
            }],
            None,
        )
    }
}

/// Check format, returns CheckResult with errors and optional corrected version
pub fn check_format(content: &str, format: &str) -> CheckResult {
    let (errors, corrected) = run_parser(content, format);
    // Never expose a repair candidate that still fails the format validator.
    let corrected = corrected.filter(|candidate| format_errors(candidate, format).is_empty());
    CheckResult {
        format: format.to_string(),
        valid: errors.is_empty(),
        errors,
        corrected,
    }
}

fn format_errors(content: &str, format: &str) -> Vec<FormatError> {
    run_parser(content, format).0
}

/// Check JSON against a JSON Schema
pub fn check_json_schema(content: &str, schema_str: &str) -> SchemaCheckResult {
    let schema_result = serde_json::from_str(schema_str);
    let content_result = serde_json::from_str::<serde_json::Value>(content);

    match (schema_result, content_result) {
        (Ok(schema), Ok(instance)) => {
            let validator = jsonschema::Validator::new(&schema);
            match validator {
                Ok(v) => {
                    let mut errors = Vec::new();
                    for e in v.iter_errors(&instance) {
                        let msg = e.to_string();
                        let path = e.instance_path.to_string();
                        let line = extract_line_from_path(content, &path);
                        errors.push(FormatError {
                            line,
                            col: None,
                            near: Some(path.clone()),
                            raw: msg.clone(),
                            friendly: format!("Schema 验证失败: {} (路径: {})", msg, path),
                        });
                    }
                    SchemaCheckResult {
                        valid: errors.is_empty(),
                        errors,
                    }
                }
                Err(e) => SchemaCheckResult {
                    valid: false,
                    errors: vec![FormatError {
                        line: None,
                        col: None,
                        near: None,
                        raw: e.to_string(),
                        friendly: format!("Schema 格式无效: {}", e),
                    }],
                },
            }
        }
        (Err(e), _) => SchemaCheckResult {
            valid: false,
            errors: vec![FormatError {
                line: None,
                col: None,
                near: None,
                raw: e.to_string(),
                friendly: format!("Schema 解析失败: {}", e),
            }],
        },
        (_, Err(e)) => SchemaCheckResult {
            valid: false,
            errors: vec![FormatError {
                line: None,
                col: None,
                near: None,
                raw: e.to_string(),
                friendly: format!("JSON 内容解析失败: {}", e),
            }],
        },
    }
}

fn extract_line_from_path(content: &str, path: &str) -> Option<u32> {
    let key = path.trim_start_matches('/');
    if let Some(last_key) = key.rsplit('/').next() {
        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(content) {
            if obj.get(last_key).is_some() {
                let search = format!("\"{}\"", last_key);
                for (i, line) in content.lines().enumerate() {
                    if line.contains(&search) {
                        return Some((i + 1) as u32);
                    }
                }
            }
        }
    }
    None
}

/// Simple auto-fix based on rules
pub fn simple_fix(content: &str, format: &str) -> String {
    if let Some(adapter) = formats::get_registry().get(format) {
        adapter
            .repair(content)
            .unwrap_or_else(|| content.to_string())
    } else {
        content.to_string()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            cmd_read_files,
            cmd_check_file,
            cmd_check_batch,
            cmd_check_schema,
            cmd_simple_fix,
            cmd_save_file,
            cmd_get_home,
            cmd_get_desktop,
            cmd_list_formats,
            cmd_detect_format,
            cmd_parse_document,
            cmd_serialize_document,
            cmd_apply_patch,
            cmd_format_document,
            cmd_validate_document,
            cmd_list_profiles,
            cmd_detect_profile,
            cmd_validate_profile,
            cmd_compute_semantic_diff,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn cmd_read_files(paths: Vec<String>) -> Result<Vec<(String, String)>, String> {
    let paths = paths.into_iter().map(PathBuf::from).collect::<Vec<_>>();
    read_files_from_paths(&paths)
}

fn read_files_from_paths(paths: &[PathBuf]) -> Result<Vec<(String, String)>, String> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    let mut total_size = 0usize;

    for path in paths {
        let raw_path = path.display().to_string();

        let metadata =
            fs::metadata(path).map_err(|e| format!("无法读取文件信息 {}: {}", raw_path, e))?;

        if !metadata.is_file() {
            return Err(format!("不是有效文件: {}", raw_path));
        }

        let size = metadata.len() as usize;
        if size > MAX_FILE_SIZE {
            return Err(format!(
                "文件过大，最大支持 {} MB: {}",
                MAX_FILE_SIZE / 1024 / 1024,
                raw_path
            ));
        }

        total_size = total_size.saturating_add(size);
        if total_size > MAX_FILE_SIZE * 2 {
            return Err(format!(
                "批量文件总大小超限，最大支持 {} MB",
                MAX_FILE_SIZE * 2 / 1024 / 1024
            ));
        }

        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("无法识别文件名: {}", raw_path))?
            .to_string();
        let content = fs::read_to_string(path)
            .map_err(|e| format!("无法读取文本文件 {}: {}", filename, e))?;

        files.push((filename, content));
    }

    Ok(files)
}

#[tauri::command]
fn cmd_check_file(content: String, filename: String) -> Result<CheckResult, String> {
    if content.len() > MAX_FILE_SIZE {
        return Err(format!(
            "文件过大，最大支持 {} MB",
            MAX_FILE_SIZE / 1024 / 1024
        ));
    }
    let format = detect_format(&filename)
        .or_else(|| detect_format_heuristic(&content))
        .unwrap_or_else(|| "unknown".into());
    Ok(check_format(&content, &format))
}

#[tauri::command]
fn cmd_check_batch(files: Vec<(String, String)>) -> Result<Vec<BatchItem>, String> {
    for (filename, content) in &files {
        if content.len() > MAX_FILE_SIZE {
            return Err(format!(
                "文件过大，最大支持 {} MB: {}",
                MAX_FILE_SIZE / 1024 / 1024,
                filename
            ));
        }
    }

    let total: usize = files.iter().map(|(_, c)| c.len()).sum();
    if total > MAX_FILE_SIZE * 2 {
        return Err(format!(
            "批量文件总大小超限，最大支持 {} MB",
            MAX_FILE_SIZE * 2 / 1024 / 1024
        ));
    }
    Ok(files
        .iter()
        .map(|(filename, content)| {
            let format = detect_format(filename)
                .or_else(|| detect_format_heuristic(content))
                .unwrap_or_else(|| "unknown".into());
            BatchItem {
                filename: filename.clone(),
                result: check_format(content, &format),
            }
        })
        .collect())
}

#[tauri::command]
fn cmd_check_schema(content: String, schema: String) -> SchemaCheckResult {
    if content.len() > MAX_FILE_SIZE {
        return SchemaCheckResult {
            valid: false,
            errors: vec![FormatError {
                line: None,
                col: None,
                near: None,
                raw: format!("文件过大，最大支持 {} MB", MAX_FILE_SIZE / 1024 / 1024),
                friendly: format!("文件过大，最大支持 {} MB", MAX_FILE_SIZE / 1024 / 1024),
            }],
        };
    }
    if schema.len() > MAX_FILE_SIZE {
        return SchemaCheckResult {
            valid: false,
            errors: vec![FormatError {
                line: None,
                col: None,
                near: None,
                raw: format!("Schema 过大，最大支持 {} MB", MAX_FILE_SIZE / 1024 / 1024),
                friendly: format!("Schema 过大，最大支持 {} MB", MAX_FILE_SIZE / 1024 / 1024),
            }],
        };
    }

    check_json_schema(&content, &schema)
}

#[tauri::command]
fn cmd_simple_fix(content: String, format: String) -> Result<String, String> {
    if content.len() > MAX_FILE_SIZE {
        return Err(format!(
            "文件过大，最大支持 {} MB",
            MAX_FILE_SIZE / 1024 / 1024
        ));
    }
    Ok(simple_fix(&content, &format))
}

#[tauri::command]
fn cmd_save_file(
    path: Option<String>,
    filename: Option<String>,
    content: String,
    directory: Option<String>,
) -> Result<String, String> {
    if content.len() > MAX_FILE_SIZE {
        return Err(format!(
            "文件过大，最大支持 {} MB",
            MAX_FILE_SIZE / 1024 / 1024
        ));
    }

    let target_path = if let Some(p) = path.filter(|s| !s.trim().is_empty()) {
        PathBuf::from(p)
    } else if let Some(fname) = filename.filter(|s| !s.trim().is_empty()) {
        let base_dir = if let Some(dir) = directory.filter(|s| !s.trim().is_empty()) {
            PathBuf::from(dir)
        } else {
            dirs::desktop_dir()
                .or_else(dirs::document_dir)
                .or_else(dirs::download_dir)
                .unwrap_or_else(|| PathBuf::from("."))
        };
        base_dir.join(fname)
    } else {
        return Err("未指定文件名或保存路径".into());
    };

    if !is_path_allowed(&target_path) {
        if target_path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        {
            return Err("路径包含非法相对父路径 (..)".into());
        }
        if !target_path.is_absolute() {
            return Err("不允许保存到该目录，仅支持桌面、文档、下载目录或绝对路径".into());
        }
    }

    // 确保父目录存在
    if let Some(parent) = target_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }
    }
    fs::write(&target_path, &content).map_err(|e| format!("保存失败: {}", e))?;
    Ok(target_path.display().to_string())
}

#[tauri::command]
fn cmd_get_home() -> Result<String, String> {
    dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "无法获取用户目录".into())
}

#[tauri::command]
fn cmd_get_desktop() -> Result<String, String> {
    dirs::desktop_dir()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "无法获取桌面目录".into())
}

#[tauri::command]
fn cmd_list_formats() -> Vec<formats::FormatDescriptor> {
    formats::get_registry().list()
}

#[tauri::command]
fn cmd_detect_format(content: Option<String>, filename: Option<String>) -> Option<String> {
    formats::get_registry().detect(filename.as_deref(), content.as_deref())
}

#[tauri::command]
fn cmd_parse_document(content: String, format: String) -> Result<document::DocumentNode, String> {
    if content.len() > MAX_FILE_SIZE {
        return Err(format!(
            "文件过大，最大支持 {} MB",
            MAX_FILE_SIZE / 1024 / 1024
        ));
    }
    let adapter = formats::get_registry()
        .get(&format)
        .ok_or_else(|| format!("不支持的格式: {}", format))?;
    adapter
        .parse(&content)
        .map_err(|e| format!("{}: {}", e.friendly, e.raw))
}

#[tauri::command]
fn cmd_serialize_document(node: document::DocumentNode, format: String) -> Result<String, String> {
    let adapter = formats::get_registry()
        .get(&format)
        .ok_or_else(|| format!("不支持的格式: {}", format))?;
    adapter
        .serialize(&node)
        .map_err(|e| format!("{}: {}", e.friendly, e.raw))
}

#[tauri::command]
fn cmd_apply_patch(
    content: String,
    format: String,
    patch: document::DocumentPatch,
) -> Result<document::PatchResult, String> {
    if content.len() > MAX_FILE_SIZE {
        return Err(format!(
            "文件过大，最大支持 {} MB",
            MAX_FILE_SIZE / 1024 / 1024
        ));
    }
    let adapter = formats::get_registry()
        .get(&format)
        .ok_or_else(|| format!("不支持的格式: {}", format))?;
    let mut doc = adapter
        .parse(&content)
        .map_err(|e| format!("解析失败: {}", e.friendly))?;

    document::apply_patch_to_node(&mut doc, &patch)?;

    let serialized = adapter
        .serialize(&doc)
        .map_err(|e| format!("序列化失败: {}", e.friendly))?;

    let errors = adapter.validate(&serialized);
    let valid = errors.is_empty();

    Ok(document::PatchResult {
        content: serialized,
        node: doc,
        valid,
        errors,
    })
}

#[tauri::command]
fn cmd_format_document(content: String, format: String) -> Result<String, String> {
    if content.len() > MAX_FILE_SIZE {
        return Err(format!(
            "文件过大，最大支持 {} MB",
            MAX_FILE_SIZE / 1024 / 1024
        ));
    }
    let adapter = formats::get_registry()
        .get(&format)
        .ok_or_else(|| format!("不支持的格式: {}", format))?;
    adapter
        .format(&content)
        .map_err(|e| format!("{}: {}", e.friendly, e.raw))
}

#[tauri::command]
fn cmd_validate_document(content: String, format: String) -> CheckResult {
    check_format(&content, &format)
}

#[tauri::command]
fn cmd_list_profiles() -> Vec<profiles::ProfileDescriptor> {
    profiles::get_profile_registry().list()
}

#[tauri::command]
fn cmd_detect_profile(
    filename: String,
    content: String,
    format: Option<String>,
    doc: Option<document::DocumentNode>,
) -> Option<String> {
    let document = if let Some(d) = doc {
        d
    } else {
        let fmt = format.unwrap_or_else(|| {
            formats::get_registry()
                .detect(Some(&filename), Some(&content))
                .unwrap_or_else(|| "json".into())
        });
        let adapter = formats::get_registry().get(&fmt)?;
        adapter.parse(&content).ok()?
    };
    profiles::get_profile_registry().detect(&filename, &content, &document)
}

#[tauri::command]
fn cmd_validate_profile(
    profile_id: String,
    content: Option<String>,
    format: Option<String>,
    doc: Option<document::DocumentNode>,
) -> Vec<profiles::ProfileDiagnostic> {
    let document = if let Some(d) = doc {
        d
    } else {
        let fmt = format.as_deref().unwrap_or("json");
        let cnt = content.as_deref().unwrap_or("");
        let Some(adapter) = formats::get_registry().get(fmt) else {
            return Vec::new();
        };
        let Ok(d) = adapter.parse(cnt) else {
            return Vec::new();
        };
        d
    };
    profiles::get_profile_registry().validate(&profile_id, &document)
}

#[tauri::command]
fn cmd_compute_semantic_diff(
    old_content: String,
    new_content: String,
    format: String,
) -> Result<document::SemanticDiffResult, String> {
    let adapter = formats::get_registry()
        .get(&format)
        .ok_or_else(|| format!("不支持的格式: {}", format))?;
    let old_doc = adapter
        .parse(&old_content)
        .map_err(|e| format!("原内容解析失败: {}", e.friendly))?;
    let new_doc = adapter
        .parse(&new_content)
        .map_err(|e| format!("新内容解析失败: {}", e.friendly))?;
    Ok(document::compute_semantic_diff(&old_doc, &new_doc))
}
#[cfg(test)]
mod tests {
    use super::parsers;

    #[test]
    fn xml_error_preview_handles_multibyte_content() {
        let content = "<root><item>中文内容</root>";
        let result = std::panic::catch_unwind(|| parsers::xml::check(content));

        assert!(result.is_ok());
        let (errors, _) = result.unwrap();
        assert!(!errors.is_empty());
    }

    #[test]
    fn csv_error_preview_handles_multibyte_content() {
        let content = "name,age\n中文内容中文内容中文内容中文内容中文内容,20,extra";
        let result = std::panic::catch_unwind(|| parsers::csv::check(content));

        assert!(result.is_ok());
        let (errors, _) = result.unwrap();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn json_fix_replaces_undefined_after_multibyte_content() {
        let content = "{\n  \"标题\": \"中文\",\n  \"value\": undefined\n}";
        let result = std::panic::catch_unwind(|| parsers::json::simple_fix(content));

        assert!(result.is_ok());
        assert!(result.unwrap().contains("\"value\": null"));
    }

    #[test]
    fn heuristic_detects_toml_section_before_json_array_fallback() {
        let content = "[server]\nhost = \"localhost\"\nport = 8080";

        assert_eq!(
            super::detect_format_heuristic(content).as_deref(),
            Some("toml")
        );
    }

    #[test]
    fn heuristic_detects_ini_section_with_bare_values() {
        let content = "[server]\nhost = localhost\nport = 8080";

        assert_eq!(
            super::detect_format_heuristic(content).as_deref(),
            Some("ini")
        );
    }

    #[test]
    fn heuristic_keeps_json_arrays_as_json() {
        let content = "[{\"name\":\"Orthos\"}]";

        assert_eq!(
            super::detect_format_heuristic(content).as_deref(),
            Some("json")
        );
    }

    #[test]
    fn batch_rejects_single_file_over_limit() {
        let files = vec![
            (
                "large.json".to_string(),
                " ".repeat(super::MAX_FILE_SIZE + 1),
            ),
            ("small.json".to_string(), "{}".to_string()),
        ];

        let err = super::cmd_check_batch(files).unwrap_err();

        assert!(err.contains("文件过大"));
        assert!(err.contains("large.json"));
    }

    #[test]
    fn schema_command_rejects_oversized_content() {
        let result = super::cmd_check_schema(" ".repeat(super::MAX_FILE_SIZE + 1), "{}".into());

        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].friendly.contains("文件过大"));
    }

    #[test]
    fn schema_command_rejects_oversized_schema() {
        let result = super::cmd_check_schema("{}".into(), " ".repeat(super::MAX_FILE_SIZE + 1));

        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].friendly.contains("Schema 过大"));
    }

    #[test]
    fn schema_reports_all_validation_errors() {
        let schema = r#"{
          "type": "object",
          "required": ["name", "port"],
          "properties": {
            "name": {"type": "string"},
            "port": {"type": "integer"}
          }
        }"#;
        let result = super::check_json_schema(r#"{"name": 42}"#, schema);

        assert!(!result.valid);
        assert!(
            result.errors.len() >= 2,
            "expected all schema errors, got {}",
            result.errors.len()
        );
    }

    #[test]
    fn schema_external_reference_is_rejected_without_network_access() {
        let schema = r#"{"$ref":"https://example.com/schema.json"}"#;
        let result = super::check_json_schema("{}", schema);

        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].friendly.contains("Schema 格式无效"));
    }

    #[test]
    fn simple_fix_command_rejects_oversized_content() {
        let err =
            super::cmd_simple_fix(" ".repeat(super::MAX_FILE_SIZE + 1), "json".into()).unwrap_err();

        assert!(err.contains("文件过大"));
    }

    #[test]
    fn save_path_allows_new_subdirectory_under_allowed_dir() {
        let Some(base) = dirs::desktop_dir()
            .or_else(dirs::document_dir)
            .or_else(dirs::download_dir)
        else {
            return;
        };
        let target = base
            .join(format!("orthos-new-save-dir-{}", std::process::id()))
            .join("fixed.json");

        assert!(super::is_path_allowed(&target));
    }

    #[test]
    fn save_path_rejects_parent_directory_traversal() {
        let Some(base) = dirs::desktop_dir()
            .or_else(dirs::document_dir)
            .or_else(dirs::download_dir)
        else {
            return;
        };
        let target = base.join("..").join("orthos-outside.txt");

        assert!(!super::is_path_allowed(&target));
    }

    #[test]
    fn dropped_file_reader_accepts_arbitrary_user_selected_path() {
        let path = std::env::temp_dir().join(format!(
            "orthos-dropped-file-reader-{}.json",
            std::process::id()
        ));
        std::fs::write(&path, "{\"name\":\"Orthos\"}").unwrap();

        let expected_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap()
            .to_string();
        let files = super::read_files_from_paths(std::slice::from_ref(&path)).unwrap();

        assert_eq!(
            files,
            vec![(expected_name, "{\"name\":\"Orthos\"}".to_string())]
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn parser_corpus_is_panic_free_and_repairs_are_valid() {
        let long_yaml = (0..128)
            .map(|i| format!("key_{i}: value_{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let corpus = [
            "",
            "\u{0}\u{0}\u{0}",
            "中文 😀 [] {} <> = : ,",
            "{broken: [1, 2,}",
            "[section\nkey value\nkey = \"unterminated",
            "<root><item>text</root>",
            "name,age\nAlice,30,extra,field",
            "key: first\nkey: second",
            long_yaml.as_str(),
        ];
        let formats = ["json", "yaml", "toml", "xml", "csv", "ini", "env"];

        for format in formats {
            for input in corpus {
                let result = std::panic::catch_unwind(|| super::check_format(input, format));
                assert!(
                    result.is_ok(),
                    "{format} parser panicked for input {:?}",
                    input
                );

                let result = result.unwrap();
                if let Some(corrected) = result.corrected {
                    let repaired = super::check_format(&corrected, format);
                    assert!(
                        repaired.valid,
                        "{format} exposed an invalid repair for input {:?}: {:?}",
                        input,
                        repaired
                            .errors
                            .iter()
                            .map(|error| &error.raw)
                            .collect::<Vec<_>>()
                    );
                }
            }
        }
    }

    #[test]
    fn csv_multiline_quoted_field() {
        let content = "name,desc,age\nAlice,\"hello\nworld\",30\nBob,\"simple\",25";
        let (errors, _) = parsers::csv::check(content);
        assert!(
            errors.is_empty(),
            "CSV multiline field was rejected: {:?}",
            errors
        );
    }

    #[test]
    fn xml_attribute_with_greater_than() {
        let content = "<root><item expr=\"a > b\" count='c > d'/></root>";
        let fixed = parsers::xml::simple_fix(content);
        let (errors, _) = parsers::xml::check(&fixed);
        assert!(
            errors.is_empty(),
            "XML attribute with > was corrupted: {}",
            fixed
        );
    }

    #[test]
    fn env_export_prefix() {
        let content = "export APP_PORT=8080\nexport DATABASE_URL=\"postgres://localhost\"\n";
        let (errors, _) = parsers::env::check(content);
        assert!(
            errors.is_empty(),
            "ENV with export prefix had errors: {:?}",
            errors
        );
        let fixed = parsers::env::simple_fix(content);
        assert!(fixed.contains("export APP_PORT=8080"));
    }

    #[test]
    fn ini_duplicate_keys_across_split_sections() {
        let content = "[server]\nhost = a\n[database]\nname = mydb\n[server]\nhost = b\n";
        let (errors, _) = parsers::ini::check(content);
        assert!(
            !errors.is_empty(),
            "INI should detect duplicate keys across split sections"
        );
        assert!(errors.iter().any(|e| e.raw.contains("重复的键名")));
    }

    #[test]
    fn heuristic_detects_ini_with_comment_header() {
        let content = "# Global configuration\n[database]\nhost = localhost\nport = 5432\n";
        assert_eq!(
            super::detect_format_heuristic(content).as_deref(),
            Some("ini")
        );
    }

    #[test]
    fn adapter_contract_all_formats_registered() {
        let list = super::formats::get_registry().list();
        assert!(list.len() >= 12);
        let ids: Vec<&str> = list.iter().map(|f| f.id.as_str()).collect();
        for expected in [
            "json",
            "yaml",
            "toml",
            "xml",
            "csv",
            "ini",
            "env",
            "jsonc",
            "json5",
            "jsonl",
            "tsv",
            "properties",
        ] {
            assert!(ids.contains(&expected), "Missing format: {}", expected);
        }
    }

    #[test]
    fn adapter_contract_json_parse_patch_serialize() {
        let input = r#"{"server": {"port": 8080, "host": "127.0.0.1"}}"#;
        let adapter = super::formats::get_registry().get("json").unwrap();
        let mut doc = adapter.parse(input).unwrap();
        assert_eq!(doc.kind, super::document::NodeKind::Object);

        // Apply SetValue patch
        let patch = super::document::DocumentPatch::SetValue {
            path: "/server/port".into(),
            value: 3000.into(),
        };
        super::document::apply_patch_to_node(&mut doc, &patch).unwrap();

        let serialized = adapter.serialize(&doc).unwrap();
        assert!(serialized.contains("3000"));
        let revalidated = adapter.validate(&serialized);
        assert!(revalidated.is_empty());
    }

    #[test]
    fn adapter_contract_csv_grid_operations() {
        let input = "name,age\nAlice,30\nBob,25";
        let adapter = super::formats::get_registry().get("csv").unwrap();
        let mut doc = adapter.parse(input).unwrap();
        assert_eq!(doc.kind, super::document::NodeKind::Table);

        // Add a row
        let patch = super::document::DocumentPatch::AddRow {
            parent_path: "/".into(),
            index: None,
            values: vec!["Charlie".into(), "35".into()],
        };
        super::document::apply_patch_to_node(&mut doc, &patch).unwrap();

        let serialized = adapter.serialize(&doc).unwrap();
        assert!(serialized.contains("Charlie,35"));
    }

    #[test]
    fn adapter_contract_ini_kv_operations() {
        let input = "[server]\nhost = localhost\nport = 8080\n";
        let adapter = super::formats::get_registry().get("ini").unwrap();
        let doc = adapter.parse(input).unwrap();
        let serialized = adapter.serialize(&doc).unwrap();
        assert!(serialized.contains("[server]"));
        assert!(serialized.contains("host = localhost"));
    }

    #[test]
    fn adapter_contract_xml_dom_operations() {
        let input = r#"<config version="1.0"><server><host>localhost</host></server></config>"#;
        let adapter = super::formats::get_registry().get("xml").unwrap();
        let doc = adapter.parse(input).unwrap();
        let serialized = adapter.serialize(&doc).unwrap();
        assert!(serialized.contains("<config"));
        assert!(serialized.contains("version=\"1.0\""));
    }

    #[test]
    fn adapter_contract_extension_formats() {
        // Test JSONC
        let jsonc_input = "{\n  // comment\n  \"name\": \"orthos\"\n}";
        let jsonc_adapter = super::formats::get_registry().get("jsonc").unwrap();
        let doc = jsonc_adapter.parse(jsonc_input).unwrap();
        assert_eq!(doc.kind, super::document::NodeKind::Object);

        // Test TSV
        let tsv_input = "col1\tcol2\nval1\tval2";
        let tsv_adapter = super::formats::get_registry().get("tsv").unwrap();
        let doc = tsv_adapter.parse(tsv_input).unwrap();
        assert_eq!(doc.kind, super::document::NodeKind::Table);

        // Test Properties
        let prop_input = "app.name = Orthos\napp.port = 8080";
        let prop_adapter = super::formats::get_registry().get("properties").unwrap();
        let doc = prop_adapter.parse(prop_input).unwrap();
        assert_eq!(doc.kind, super::document::NodeKind::Object);

        // Test EditorConfig
        let ec_input = "root = true\n\n[*]\nindent_style = space\n";
        let ec_adapter = super::formats::get_registry().get("editorconfig").unwrap();
        let doc = ec_adapter.parse(ec_input).unwrap();
        assert_eq!(doc.kind, super::document::NodeKind::Object);

        // Test GitConfig
        let gc_input = "[user]\nname = Ryan\nemail = ryan@example.com\n";
        let gc_adapter = super::formats::get_registry().get("gitconfig").unwrap();
        let doc = gc_adapter.parse(gc_input).unwrap();
        assert_eq!(doc.kind, super::document::NodeKind::Object);

        // Test HCL
        let hcl_input = "variable \"region\" {\n  default = \"us-east-1\"\n}\n";
        let hcl_adapter = super::formats::get_registry().get("hcl").unwrap();
        let doc = hcl_adapter.parse(hcl_input).unwrap();
        assert_eq!(doc.kind, super::document::NodeKind::Object);
    }

    #[test]
    fn profile_contract_package_json_and_docker_compose() {
        // package.json detection & semantic validation
        let pkg_content = r#"{
          "name": "MyPackage",
          "version": "1.0",
          "dependencies": {"react": "^18.0.0"},
          "devDependencies": {"react": "^18.0.0"}
        }"#;
        let json_adapter = super::formats::get_registry().get("json").unwrap();
        let pkg_doc = json_adapter.parse(pkg_content).unwrap();

        let detected =
            super::profiles::get_profile_registry().detect("package.json", pkg_content, &pkg_doc);
        assert_eq!(detected.as_deref(), Some("package.json"));

        let diags = super::profiles::get_profile_registry().validate("package.json", &pkg_doc);
        assert!(diags.iter().any(|d| d.message.contains("大写字母")));
        assert!(diags.iter().any(|d| d
            .message
            .contains("同时出现在 dependencies 和 devDependencies")));

        // docker-compose duplicate port check
        let compose_content = r#"
services:
  web:
    image: nginx
    ports:
      - "8080:80"
  api:
    image: node
    ports:
      - "8080:3000"
"#;
        let yaml_adapter = super::formats::get_registry().get("yaml").unwrap();
        let compose_doc = yaml_adapter.parse(compose_content).unwrap();
        let compose_detected = super::profiles::get_profile_registry().detect(
            "docker-compose.yml",
            compose_content,
            &compose_doc,
        );
        assert_eq!(compose_detected.as_deref(), Some("docker-compose"));

        let compose_diags =
            super::profiles::get_profile_registry().validate("docker-compose", &compose_doc);
        assert!(compose_diags.iter().any(|d| d.message.contains("冲突")));
    }

    #[test]
    fn semantic_diff_computation() {
        let old_content = r#"{"server": {"port": 8080, "host": "localhost"}, "debug": true}"#;
        let new_content = r#"{"server": {"port": 3000, "host": "localhost"}, "extra": "new"}"#;

        let diff =
            super::cmd_compute_semantic_diff(old_content.into(), new_content.into(), "json".into())
                .unwrap();

        assert_eq!(diff.modified_count, 1); // port changed
        assert_eq!(diff.added_count, 1); // extra added
        assert_eq!(diff.removed_count, 1); // debug removed
        assert_eq!(diff.items.len(), 3);
    }

    #[test]
    fn cmd_save_file_supports_filename_and_directory() {
        let temp_dir =
            std::env::temp_dir().join(format!("orthos-save-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let filename = "saved-config.json";
        let content = "{\"test\": true}";

        let saved = super::cmd_save_file(
            None,
            Some(filename.into()),
            content.into(),
            Some(temp_dir.display().to_string()),
        )
        .unwrap();

        let expected_path = temp_dir.join(filename);
        assert_eq!(saved, expected_path.display().to_string());
        assert_eq!(std::fs::read_to_string(&expected_path).unwrap(), content);

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn cmd_save_file_supports_direct_path() {
        let temp_file =
            std::env::temp_dir().join(format!("orthos-direct-save-{}.json", std::process::id()));
        let content = "{\"direct\": true}";

        let saved = super::cmd_save_file(
            Some(temp_file.display().to_string()),
            None,
            content.into(),
            None,
        )
        .unwrap();

        assert_eq!(saved, temp_file.display().to_string());
        assert_eq!(std::fs::read_to_string(&temp_file).unwrap(), content);

        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn cmd_detect_and_validate_profile_flexible_args() {
        let pkg_content = r#"{"name": "BadPackage", "version": "1.0.0"}"#;
        let detected =
            super::cmd_detect_profile("package.json".into(), pkg_content.into(), None, None);
        assert_eq!(detected.as_deref(), Some("package.json"));

        let diags = super::cmd_validate_profile(
            "package.json".into(),
            Some(pkg_content.into()),
            Some("json".into()),
            None,
        );
        assert!(diags.iter().any(|d| d.message.contains("大写字母")));
    }

    #[test]
    fn detect_format_recognizes_extended_formats() {
        assert_eq!(
            super::detect_format("settings.jsonc").as_deref(),
            Some("jsonc")
        );
        assert_eq!(super::detect_format("app.json5").as_deref(), Some("json5"));
        assert_eq!(super::detect_format("data.tsv").as_deref(), Some("tsv"));
        assert_eq!(
            super::detect_format("config.properties").as_deref(),
            Some("properties")
        );
        assert_eq!(
            super::detect_format(".editorconfig").as_deref(),
            Some("editorconfig")
        );
        assert_eq!(
            super::detect_format(".gitconfig").as_deref(),
            Some("gitconfig")
        );
        assert_eq!(super::detect_format("main.tf").as_deref(), Some("hcl"));
        assert_eq!(
            super::detect_format("records.ndjson").as_deref(),
            Some("jsonl")
        );
    }

    #[test]
    fn xml_sibling_path_independence_and_patch() {
        let xml_input = "<root><item>First</item><item>Second</item></root>";
        let adapter = super::formats::get_registry().get("xml").unwrap();
        let mut doc = adapter.parse(xml_input).unwrap();

        // Ensure siblings have distinct paths
        let children = doc.children.as_ref().unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].path, "/item#0");
        assert_eq!(children[1].path, "/item#1");

        // Remove only the second item
        let patch = super::document::DocumentPatch::RemoveNode {
            path: "/item#1".into(),
        };
        super::document::apply_patch_to_node(&mut doc, &patch).unwrap();

        assert_eq!(doc.children.as_ref().unwrap().len(), 1);
        let serialized = adapter.serialize(&doc).unwrap();
        assert!(serialized.contains("First"));
        assert!(!serialized.contains("Second"));
    }
}
