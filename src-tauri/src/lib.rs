mod parsers;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
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
    let name = filename.to_lowercase();
    if name.ends_with(".json") {
        Some("json".into())
    } else if name.ends_with(".yaml") || name.ends_with(".yml") {
        Some("yaml".into())
    } else if name.ends_with(".toml") {
        Some("toml".into())
    } else if name.ends_with(".xml") {
        Some("xml".into())
    } else if name.ends_with(".csv") {
        Some("csv".into())
    } else if name.ends_with(".ini") {
        Some("ini".into())
    } else if name.ends_with(".env") {
        Some("env".into())
    } else {
        None
    }
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
    } else if trimmed.starts_with('#') && trimmed.contains('=') {
        Some("env".into())
    } else if trimmed.lines().any(|l| {
        let t = l.trim();
        t.starts_with('[') && t.contains(']')
    }) && trimmed.lines().any(|l| l.contains('='))
    {
        Some("toml".into())
    } else if trimmed.lines().any(|l| {
        let t = l.trim();
        t.contains(": ") && !t.starts_with('#')
    }) {
        Some("yaml".into())
    } else if trimmed.contains(',') && trimmed.lines().count() > 1 {
        Some("csv".into())
    } else if trimmed.lines().any(|l| {
        let t = l.trim();
        t.starts_with('[') && t.contains(']')
    }) {
        Some("ini".into())
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

/// Check format, returns CheckResult with errors and optional corrected version
pub fn check_format(content: &str, format: &str) -> CheckResult {
    let (errors, corrected) = match format {
        "json" => parsers::json::check(content),
        "yaml" => parsers::yaml::check(content),
        "toml" => parsers::toml::check(content),
        "xml" => parsers::xml::check(content),
        "csv" => parsers::csv::check(content),
        "ini" => parsers::ini::check(content),
        "env" => parsers::env::check(content),
        _ => (
            vec![FormatError {
                line: None,
                col: None,
                near: None,
                raw: format!("不支持的格式: {}", format),
                friendly: format!("暂不支持 {} 格式的校验", format),
            }],
            None,
        ),
    };
    // Never expose a repair candidate that still fails the format validator.
    // Individual parsers may produce a best-effort candidate while repairing
    // multiple independent issues; the UI must not present that candidate as
    // a downloadable fix.
    let corrected = corrected.filter(|candidate| format_errors(candidate, format).is_empty());
    CheckResult {
        format: format.to_string(),
        valid: errors.is_empty(),
        errors,
        corrected,
    }
}

fn format_errors(content: &str, format: &str) -> Vec<FormatError> {
    match format {
        "json" => parsers::json::check(content).0,
        "yaml" => parsers::yaml::check(content).0,
        "toml" => parsers::toml::check(content).0,
        "xml" => parsers::xml::check(content).0,
        "csv" => parsers::csv::check(content).0,
        "ini" => parsers::ini::check(content).0,
        "env" => parsers::env::check(content).0,
        _ => vec![FormatError {
            line: None,
            col: None,
            near: None,
            raw: format!("不支持的格式: {}", format),
            friendly: format!("暂不支持 {} 格式的校验", format),
        }],
    }
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
    match format {
        "json" => parsers::json::simple_fix(content),
        "yaml" => parsers::yaml::simple_fix(content),
        "toml" => parsers::toml::simple_fix(content),
        "xml" => parsers::xml::simple_fix(content),
        "csv" => parsers::csv::simple_fix(content),
        "ini" => parsers::ini::simple_fix(content),
        "env" => parsers::env::simple_fix(content),
        _ => content.to_string(),
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
fn cmd_save_file(path: String, content: String) -> Result<(), String> {
    if content.len() > MAX_FILE_SIZE {
        return Err(format!(
            "文件过大，最大支持 {} MB",
            MAX_FILE_SIZE / 1024 / 1024
        ));
    }
    let target = Path::new(&path);
    if !is_path_allowed(target) {
        return Err("不允许保存到该目录，仅支持桌面、文档、下载目录".into());
    }
    // 确保父目录存在
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    fs::write(&path, content).map_err(|e| format!("保存失败: {}", e))
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
}
