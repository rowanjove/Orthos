use crate::FormatError;

pub fn check(content: &str) -> (Vec<FormatError>, Option<String>) {
    let content = content.trim_start_matches('\u{feff}');
    let mut errors = Vec::new();

    match serde_json::from_str::<serde_json::Value>(content) {
        Ok(_) => {}
        Err(e) => {
            let raw_msg = e.to_string();
            let (line, col) = parse_position(&raw_msg);
            let near = line.map(|l| {
                let lines: Vec<&str> = content.lines().collect();
                if let Some(line_str) = lines.get((l - 1) as usize) {
                    let c = col.unwrap_or(1) as usize;
                    let start = c.saturating_sub(10);
                    let end = std::cmp::min(line_str.len(), c + 30);
                    // 确保切片在字符边界上
                    let safe_start = find_char_boundary_start(line_str, start);
                    let safe_end = find_char_boundary_end(line_str, end);
                    line_str[safe_start..safe_end].trim().to_string()
                } else {
                    String::new()
                }
            });
            let friendly = friendly_message(&raw_msg, content);
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

/// 找到安全的 UTF-8 切片起始位置（不落在多字节字符中间）
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

/// 找到安全的 UTF-8 切片结束位置
fn find_char_boundary_end(s: &str, pos: usize) -> usize {
    let mut i = std::cmp::min(pos, s.len());
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

/// 从 serde 错误消息中解析行列号
/// serde_json 格式: "line N column M ..."
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

fn friendly_message(raw: &str, content: &str) -> String {
    if raw.contains("expected value") {
        let lines: Vec<&str> = content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.ends_with(',') {
                let next_non_empty = lines.iter().skip(i + 1).find(|l| !l.trim().is_empty());
                if let Some(next) = next_non_empty {
                    let next_trimmed = next.trim();
                    if next_trimmed.starts_with('}') || next_trimmed.starts_with(']') {
                        return "多余的逗号——JSON 最后一项后面不能有逗号".into();
                    }
                }
            }
        }
        return "JSON 语法错误，可能缺少引号、括号或值".into();
    }
    if raw.contains("unexpected end") || raw.contains("EOF") {
        return "内容不完整，可能有未闭合的 { 或 [".into();
    }
    if raw.contains("expected") {
        return "内容不完整，可能缺少括号或引号".into();
    }
    format!("JSON 格式错误: {}", raw)
}

/// 全面的 JSON 修正，覆盖以下错误类型：
/// 1. 末尾多余逗号  }  ,  →  }
/// 2. 缺失逗号     }  "key"  →  }, "key"
/// 3. 单引号替换为双引号  'key' → "key"
/// 4. 无引号键名    { key: "value" } → { "key": "value" }
/// 5. undefined → null（仅字符串外部）
/// 6. 注释移除   // comment  和  /* comment */
/// 7. 缺失闭合括号/花括号  补全
pub fn simple_fix(content: &str) -> String {
    // Pass 0: 移除 BOM
    let content = content.trim_start_matches('\u{feff}');
    // Pass 1: 移除注释
    let pass1 = remove_json_comments(content);
    // Pass 2: 单引号 → 双引号
    let pass2 = fix_single_quotes(&pass1);
    // Pass 3: 无引号键名加引号
    let pass3 = fix_unquoted_keys(&pass2);
    // Pass 4: undefined → null（仅替换字符串外部的 undefined）
    let pass4 = replace_undefined_outside_strings(&pass3);
    // Pass 5: 删除末尾多余逗号
    let pass5 = remove_trailing_commas(&pass4);
    // Pass 6: 补缺失逗号
    let pass6 = add_missing_commas(&pass5);
    // Pass 7: 补缺失闭合括号
    let pass7 = fix_unclosed_brackets(&pass6);

    // 尝试解析并重新格式化
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&pass7) {
        if let Ok(pretty) = serde_json::to_string_pretty(&val) {
            return pretty;
        }
    }
    // 回退到 pass6
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&pass6) {
        if let Ok(pretty) = serde_json::to_string_pretty(&val) {
            return pretty;
        }
    }
    pass7
}

/// 仅替换字符串外部的 undefined → null
fn replace_undefined_outside_strings(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let undefined: Vec<char> = "undefined".chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_string = false;

    while i < len {
        if chars[i] == '\\' && in_string && i + 1 < len {
            result.push(chars[i]);
            result.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if chars[i] == '"' {
            in_string = !in_string;
            result.push(chars[i]);
            i += 1;
            continue;
        }
        if in_string {
            result.push(chars[i]);
            i += 1;
            continue;
        }
        // 检查是否是 "undefined"（字符串外部）
        if i + undefined.len() <= len && chars[i..].starts_with(&undefined) {
            // 确保不是某个标识符的一部分（前面和后面不是字母数字或下划线）
            let before_ok = i == 0 || (!chars[i - 1].is_alphanumeric() && chars[i - 1] != '_');
            let after_idx = i + undefined.len();
            let after_ok = after_idx >= len
                || (!chars[after_idx].is_alphanumeric() && chars[after_idx] != '_');
            if before_ok && after_ok {
                result.push_str("null");
                i += undefined.len();
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// 移除 // 和 /* */ 注释（JSON 标准不支持注释，但很多配置文件会用）
fn remove_json_comments(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_string = false;

    while i < len {
        if chars[i] == '\\' && in_string && i + 1 < len {
            // 转义字符：跳过下一个字符
            result.push(chars[i]);
            result.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if chars[i] == '"' {
            in_string = !in_string;
            result.push(chars[i]);
            i += 1;
            continue;
        }
        if in_string {
            result.push(chars[i]);
            i += 1;
            continue;
        }
        // // 行注释
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '/' {
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        // /* 块注释 */
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i += 2; // skip */
            continue;
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// 单引号字符串 → 双引号字符串
fn fix_single_quotes(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // 双引号字符串保持不变
        if chars[i] == '"' {
            result.push('"');
            i += 1;
            while i < len && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < len {
                    result.push(chars[i]);
                    result.push(chars[i + 1]);
                    i += 2;
                } else {
                    result.push(chars[i]);
                    i += 1;
                }
            }
            if i < len {
                result.push('"');
                i += 1;
            }
            continue;
        }
        // 单引号字符串 → 双引号
        if chars[i] == '\'' {
            result.push('"');
            i += 1;
            while i < len && chars[i] != '\'' {
                if chars[i] == '\\' && i + 1 < len {
                    // 转义单引号 → 直接加引号
                    if chars[i + 1] == '\'' {
                        result.push('\'');
                        i += 2;
                    } else {
                        result.push(chars[i]);
                        result.push(chars[i + 1]);
                        i += 2;
                    }
                } else if chars[i] == '"' {
                    // 双引号在单引号字符串中需要转义
                    result.push('\\');
                    result.push('"');
                    i += 1;
                } else {
                    result.push(chars[i]);
                    i += 1;
                }
            }
            result.push('"');
            if i < len {
                i += 1;
            }
            continue;
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// 无引号的键名加双引号   { key: "value" } → { "key": "value" }
fn fix_unquoted_keys(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();

    for line in &lines {
        let trimmed = line.trim_start();
        let indent: String = line
            .chars()
            .take_while(|c| *c == ' ' || *c == '\t')
            .collect();

        // 跳过 { 或 [ 前缀，提取后面的键名
        let (prefix, rest_of_trimmed) = if let Some(rest) = trimmed.strip_prefix('{') {
            ("{", rest)
        } else if let Some(rest) = trimmed.strip_prefix('[') {
            ("[", rest)
        } else {
            ("", trimmed)
        };

        // 尝试修复当前行的无引号键名
        let fixed = fix_unquoted_key_in_text(rest_of_trimmed);
        result.push(format!("{}{}{}", indent, prefix, fixed));
    }

    result.join("\n")
}

/// 修复文本中的无引号键名（支持同行多个键值对）
fn fix_unquoted_key_in_text(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;
    let mut in_string = false;
    let mut string_char = '"';

    while i < len {
        // 跳过字符串内容
        if in_string {
            if chars[i] == '\\' && i + 1 < len {
                result.push(chars[i]);
                result.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if chars[i] == string_char {
                in_string = false;
            }
            result.push(chars[i]);
            i += 1;
            continue;
        }

        // 进入字符串
        if chars[i] == '"' || chars[i] == '\'' {
            in_string = true;
            string_char = chars[i];
            result.push(chars[i]);
            i += 1;
            continue;
        }

        // 检查是否是无引号键名: identifier followed by ':'
        if (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '-')
            && (i == 0 || matches!(chars[i - 1], ':' | ',' | '{' | '[' | ' '))
        {
            let start = i;
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '-') {
                i += 1;
            }
            let key: String = chars[start..i].iter().collect();
            // 跳过空格
            let mut j = i;
            while j < len && chars[j] == ' ' {
                j += 1;
            }
            // 检查后面是否是冒号
            if j < len
                && chars[j] == ':'
                && !key.is_empty()
                && key != "true"
                && key != "false"
                && key != "null"
            {
                result.push('"');
                result.push_str(&key);
                result.push('"');
                i = j; // 冒号会在下次循环中被复制
            } else {
                // 不是键名，原样输出
                result.push_str(&key);
            }
            continue;
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}

/// 删除末尾多余逗号: ,} → }  ,] → ]
fn remove_trailing_commas(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '\\' && i + 1 < len {
            result.push(chars[i]);
            result.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if chars[i] == '"' {
            result.push(chars[i]);
            i += 1;
            while i < len && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < len {
                    result.push(chars[i]);
                    result.push(chars[i + 1]);
                    i += 2;
                } else {
                    result.push(chars[i]);
                    i += 1;
                }
            }
            if i < len {
                result.push(chars[i]);
                i += 1;
            }
            continue;
        }
        if chars[i] == ',' {
            let mut j = i + 1;
            while j < len
                && (chars[j] == ' ' || chars[j] == '\n' || chars[j] == '\r' || chars[j] == '\t')
            {
                j += 1;
            }
            if j < len && (chars[j] == '}' || chars[j] == ']') {
                i += 1;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// 补缺失逗号
fn add_missing_commas(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result: Vec<String> = Vec::new();

    for line in &lines {
        let trimmed = line.trim();

        if !result.is_empty() {
            // 找到上一个非空行
            if let Some(last) = result.iter().rev().find(|l| !l.trim().is_empty()) {
                let last_trimmed = last.trim();
                let last_is_value_end = last_trimmed.ends_with('}')
                    || last_trimmed.ends_with(']')
                    || last_trimmed.ends_with('"')
                    || last_trimmed.ends_with("true")
                    || last_trimmed.ends_with("false")
                    || last_trimmed.ends_with("null")
                    || last_trimmed
                        .chars()
                        .last()
                        .is_some_and(|c| c.is_ascii_digit());

                let current_is_structure = trimmed.starts_with('{')
                    || trimmed.starts_with('[')
                    || trimmed.starts_with('"');

                // 上一行是值结尾，当前行是新结构或键 → 补逗号
                if last_is_value_end && current_is_structure {
                    if let Some(last_mut) = result.iter_mut().rev().find(|l| !l.trim().is_empty()) {
                        last_mut.push(',');
                    }
                }
            }
        }

        result.push(line.to_string());
    }

    result.join("\n")
}

/// 补缺失闭合括号: 用栈跟踪未闭合的括号，在正确位置插入闭合符号
fn fix_unclosed_brackets(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_string = false;
    let mut stack: Vec<char> = Vec::new(); // 未闭合的开括号
    let mut insertions: Vec<(usize, char)> = Vec::new(); // (position, char_to_insert)

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
        let mut advance = true;
        match chars[i] {
            '{' | '[' => {
                stack.push(chars[i]);
            }
            '}' => {
                if let Some(&top) = stack.last() {
                    if top == '{' {
                        stack.pop();
                    } else {
                        // 不匹配：插入对应的闭合符号，然后重新处理当前字符
                        let close = if top == '[' { ']' } else { '}' };
                        insertions.push((i, close));
                        stack.pop();
                        advance = false; // 不前进，重新处理当前 }
                    }
                }
                // 栈空时 } 无法匹配，跳过
            }
            ']' => {
                if let Some(&top) = stack.last() {
                    if top == '[' {
                        stack.pop();
                    } else {
                        let close = if top == '{' { '}' } else { ']' };
                        insertions.push((i, close));
                        stack.pop();
                        advance = false;
                    }
                }
            }
            _ => {}
        }
        if advance {
            i += 1;
        }
    }

    // 处理剩余未闭合的括号
    for &open in stack.iter().rev() {
        let close = if open == '{' { '}' } else { ']' };
        insertions.push((len, close));
    }

    if insertions.is_empty() {
        return content.to_string();
    }

    // 按位置正序插入，累计偏移量修正位置
    insertions.sort_by_key(|&(pos, _)| pos);
    let mut result: Vec<char> = chars;
    for (offset, (pos, ch)) in insertions.into_iter().enumerate() {
        result.insert(pos + offset, ch);
    }
    result.into_iter().collect()
}
