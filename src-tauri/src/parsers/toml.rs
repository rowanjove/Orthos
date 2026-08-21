use crate::FormatError;

pub fn check(content: &str) -> (Vec<FormatError>, Option<String>) {
    let content = content.trim_start_matches('\u{feff}');
    let mut errors = Vec::new();

    match content.parse::<toml::Table>() {
        Ok(_) => {}
        Err(e) => {
            let raw_msg = format!("{}", e);
            let (line, col) = parse_position(&raw_msg);
            let near = line.map(|l| {
                let lines: Vec<&str> = content.lines().collect();
                if let Some(line_str) = lines.get((l - 1) as usize) {
                    let c = col.unwrap_or(1) as usize;
                    let start = c.saturating_sub(10);
                    let end = std::cmp::min(line_str.len(), c + 30);
                    let safe_start = find_char_boundary_start(line_str, start);
                    let safe_end = find_char_boundary_end(line_str, end);
                    line_str[safe_start..safe_end].trim().to_string()
                } else {
                    String::new()
                }
            });
            let friendly = friendly_message(&raw_msg);
            errors.push(FormatError {
                line,
                col,
                near,
                raw: raw_msg,
                friendly,
            });
        }
    }

    let corrected = if !errors.is_empty() {
        let fixed = simple_fix(content);
        if fixed != content {
            Some(fixed)
        } else {
            None
        }
    } else {
        None
    };

    (errors, corrected)
}

fn find_char_boundary_start(s: &str, pos: usize) -> usize {
    if pos >= s.len() {
        return s.len();
    }
    let mut i = pos;
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn find_char_boundary_end(s: &str, pos: usize) -> usize {
    let mut i = std::cmp::min(pos, s.len());
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

fn parse_position(msg: &str) -> (Option<u32>, Option<u32>) {
    let mut line = None;
    let mut col = None;
    let tokens: Vec<&str> = msg.split_whitespace().collect();
    for i in 0..tokens.len() {
        if tokens[i] == "line" {
            if let Some(next) = tokens.get(i + 1) {
                line = next.parse().ok();
            }
        }
        if tokens[i] == "column" {
            if let Some(next) = tokens.get(i + 1) {
                col = next.parse().ok();
            }
        }
    }
    (line, col)
}

fn friendly_message(raw: &str) -> String {
    let lower = raw.to_lowercase();
    if lower.contains("expected") && lower.contains("key") {
        return "缺少键名或键名格式不正确".into();
    }
    if lower.contains("expected") && lower.contains("=") {
        return "缺少等号，格式应为 key = value".into();
    }
    if lower.contains("invalid") && lower.contains("string") {
        return "字符串引号不配对或包含非法字符".into();
    }
    if lower.contains("invalid") && lower.contains("number") {
        return "数字格式不正确".into();
    }
    if lower.contains("duplicate") {
        return "同一 section 内有重复的键名".into();
    }
    if lower.contains("expected") && lower.contains("newline") {
        return "每行应以键值对或 section 头开头".into();
    }
    if lower.contains("invalid") && lower.contains("character") {
        return "包含非法字符，TOML 字符串需要使用引号包裹".into();
    }
    if lower.contains("expected") && lower.contains("]") {
        return "section 头未闭合，缺少 ]".into();
    }
    if lower.contains("invalid") && lower.contains("date") {
        return "日期格式不正确".into();
    }
    format!("TOML 格式错误: {}", raw)
}

/// 全面的 TOML 修正，覆盖以下错误类型：
/// 1. 重复键名保留原样，由校验阶段报告（避免静默丢失配置）
/// 2. 未闭合引号补全  "hello → "hello"
/// 3. 缺失等号  key value → key = value
/// 4. 等号两侧空格规范化  key=value → key = value
/// 5. 未闭合 section 头  [section → [section]
/// 6. 键名缺少引号（包含特殊字符时）
/// 7. 重新序列化输出
pub fn simple_fix(content: &str) -> String {
    let content = content.trim_start_matches('\u{feff}');
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();

    for line in &lines {
        let trimmed = line.trim();

        // 空行和注释行保持不变
        if trimmed.is_empty() || trimmed.starts_with('#') {
            result.push(line.to_string());
            continue;
        }

        // 修正 section 头: [section → [section]
        if trimmed.starts_with('[') && !trimmed.starts_with("[[") {
            if trimmed.contains(']') {
                result.push(line.to_string());
            } else {
                let section_name = trimmed[1..].trim();
                result.push(format!("[{}]", section_name));
            }
            continue;
        }

        // 修正 array of tables: [[section → [[section]]
        if let Some(rest) = trimmed.strip_prefix("[[") {
            if trimmed.ends_with("]]") {
                result.push(line.to_string());
            } else {
                let section_name = rest.trim().trim_end_matches(']');
                result.push(format!("[[{}]]", section_name));
            }
            continue;
        }

        // 键值对处理
        if let Some(eq_pos) = find_equal_sign(trimmed) {
            let key = trimmed[..eq_pos].trim();
            let value = trimmed[eq_pos + 1..].trim();

            // 修正未闭合引号和括号
            let fixed_value = fix_unclosed_quotes(value);
            let fixed_value = fix_unclosed_brackets(&fixed_value);

            // 修正等号两侧空格
            let fixed_key = if key.contains(' ') && !key.starts_with('"') && !key.starts_with('\'')
            {
                format!("\"{}\"", key)
            } else {
                key.to_string()
            };

            result.push(format!("{} = {}", fixed_key, fixed_value));
            continue;
        }

        // 没有等号的非空行 → 尝试修复
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            let key_end = trimmed
                .char_indices()
                .take_while(|(_, c)| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
                .map(|(i, c)| i + c.len_utf8())
                .last()
                .unwrap_or(0);
            let key = &trimmed[..key_end];
            let value = trimmed[key_end..].trim();
            if !key.is_empty() && !value.is_empty() {
                let fixed_key =
                    if key.contains(' ') && !key.starts_with('"') && !key.starts_with('\'') {
                        format!("\"{}\"", key)
                    } else {
                        key.to_string()
                    };
                let fixed_value = fix_unclosed_quotes(value);
                result.push(format!("{} = {}", fixed_key, fixed_value));
                continue;
            }
        }

        result.push(line.to_string());
    }

    let fixed = result.join("\n");

    // 尝试解析并重新序列化
    if let Ok(val) = fixed.parse::<toml::Table>() {
        if let Ok(pretty) = toml::to_string_pretty(&val) {
            return pretty;
        }
    }
    fixed
}

/// 查找等号位置（跳过字符串内的等号）
fn find_equal_sign(s: &str) -> Option<usize> {
    let mut in_string = false;
    let mut string_char = '"';
    let mut escaped = false;

    for (byte_index, ch) in s.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == string_char {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' | '\'' => {
                in_string = true;
                string_char = ch;
            }
            '=' => return Some(byte_index),
            _ => {}
        }
    }
    None
}

/// 修正未闭合的引号
fn fix_unclosed_quotes(value: &str) -> String {
    if value.is_empty() {
        return value.to_string();
    }

    // 处理多行字符串
    if value.starts_with("\"\"\"") || value.starts_with("'''") {
        let quote_char = value.chars().next().unwrap();
        let triple = format!("{c}{c}{c}", c = quote_char);
        if value.ends_with(&triple) && value.len() > 6 {
            return value.to_string();
        }
        // 获取第 3 个引号之后的字节偏移（跳过开头的 3 个引号）
        let inner_start = value
            .char_indices()
            .nth(2)
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(value.len());
        let inner = &value[inner_start..];
        if !inner.is_empty() {
            return format!("\"\"\"{}\"\"\"", inner);
        }
        return "\"\"\"\"\"\"".to_string();
    }

    // 普通字符串
    if value.starts_with('"') && !value.ends_with('"') {
        let inner = &value[1..];
        let unescaped = inner.replace("\\\"", "");
        if !unescaped.contains('"') {
            return format!("{}\"", value);
        }
    }
    if value.starts_with('\'') && !value.ends_with('\'') {
        let inner = &value[1..];
        if !inner.contains('\'') {
            return format!("{}'", value);
        }
    }

    value.to_string()
}

/// 修正未闭合的方括号和花括号
fn fix_unclosed_brackets(value: &str) -> String {
    let mut bracket_count = 0i32; // [
    let mut brace_count = 0i32; // {
    let mut in_string = false;
    let chars: Vec<char> = value.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '\\' && in_string && i + 1 < len {
            i += 2;
            continue;
        }
        if chars[i] == '"' {
            in_string = !in_string;
            i += 1;
            continue;
        }
        if in_string {
            i += 1;
            continue;
        }
        match chars[i] {
            '[' => bracket_count += 1,
            ']' => bracket_count -= 1,
            '{' => brace_count += 1,
            '}' => brace_count -= 1,
            _ => {}
        }
        i += 1;
    }

    let mut result = value.to_string();
    for _ in 0..bracket_count.max(0) {
        result.push(']');
    }
    for _ in 0..brace_count.max(0) {
        result.push('}');
    }
    result
}
