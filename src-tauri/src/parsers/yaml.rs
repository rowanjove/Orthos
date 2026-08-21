use crate::FormatError;

pub fn check(content: &str) -> (Vec<FormatError>, Option<String>) {
    let content = content.trim_start_matches('\u{feff}');
    let mut errors = Vec::new();

    match serde_yaml::from_str::<serde_yaml::Value>(content) {
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
    if lower.contains("bad indentation") || lower.contains("indentation") {
        return "缩进错误：必须用空格，同级必须对齐".into();
    }
    if lower.contains("mapping values") {
        return "冒号后缺少空格，格式应为 key: value".into();
    }
    if lower.contains("duplicat") {
        return "同一层级有重复的键名".into();
    }
    if lower.contains("tab") {
        return "使用了 Tab 缩进，请替换为空格".into();
    }
    if lower.contains("found unexpected end of stream") || lower.contains("while parsing") {
        return "内容不完整，可能缺少冒号或缩进不正确".into();
    }
    if lower.contains("found character that cannot start") {
        return "发现不合法的字符".into();
    }
    if lower.contains("mapping") && lower.contains("not allowed") {
        return "映射格式不正确".into();
    }
    format!("YAML 格式错误: {}", raw)
}

/// 全面的 YAML 修正，覆盖以下错误类型：
/// 1. Tab 缩进 → 空格
/// 2. 冒号后缺空格  key:value → key: value
/// 3. 重复键名保留原样，由校验阶段报告（避免静默丢失配置）
/// 4. 缩进修正（统一为 2 空格倍数）
/// 5. 空列表项修正  -  → - null
/// 6. 流式列表空格  [1,2,3] → [1, 2, 3]
/// 7. 流式映射空格  {a:1} → {a: 1}
/// 8. 行尾空格移除
/// 9. 错误的列表格式  -value → - value
/// 10. 重新序列化输出
pub fn simple_fix(content: &str) -> String {
    // Pass 0: 移除 BOM
    let content = content.trim_start_matches('\u{feff}');
    // Pass 1: Tab → 对齐到 4 空格停止位
    let pass1 = expand_tabs_to_indent(content, 4);

    // Pass 2: 逐行修正
    let lines: Vec<&str> = pass1.lines().collect();
    let mut result = Vec::new();
    // 跟踪列表项缩进：当 -X 被修正为 - X 时，后续行需要增加缩进
    let mut list_indent: Option<usize> = None;

    for line in &lines {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            list_indent = None;
            result.push(String::new());
            continue;
        }

        let indent: String = trimmed.chars().take_while(|c| *c == ' ').collect();
        let content_part = &trimmed[indent.len()..];

        // 如果上一行是修正过的列表项，且当前行是同级新列表项，清除旧状态
        if let Some(li) = list_indent {
            let is_new_list_item = content_part.starts_with("- ")
                || (content_part.starts_with('-') && content_part.len() > 1);
            if is_new_list_item {
                list_indent = None;
            } else if indent.len() < li + 2 {
                // 续行：缩进不够，增加缩进
                let new_indent = " ".repeat(li + 2);
                list_indent = None;
                let fixed_line = fix_yaml_line(content_part, &new_indent);
                result.push(fixed_line);
                continue;
            } else {
                list_indent = None;
            }
        }

        // Fix: 空列表项 - → - null
        if content_part == "-" {
            result.push(format!("{}- null", indent));
            continue;
        }

        // Fix: -value → - value (列表项后缺少空格，但排除负数 -3.14)
        if content_part.starts_with('-')
            && !content_part.starts_with("- ")
            && content_part.len() > 1
            && !content_part.as_bytes()[1].is_ascii_digit()
            && content_part.as_bytes()[1] != b'.'
        {
            let rest = &content_part[1..];
            result.push(format!("{}- {}", indent, rest));
            list_indent = Some(indent.len());
            continue;
        }

        let fixed_line = fix_yaml_line(content_part, &indent);
        result.push(fixed_line);
    }

    let fixed = result.join("\n");

    // Pass 3: 尝试解析并重新序列化。重复键等语义冲突不做猜测，
    // 保留原文并交给校验结果提示用户处理。
    if let Ok(val) = serde_yaml::from_str::<serde_yaml::Value>(&fixed) {
        if let Ok(pretty) = serde_yaml::to_string(&val) {
            return pretty;
        }
    }
    fixed
}

/// 修正单行 YAML 内容的冒号空格和流式格式
fn fix_yaml_line(content_part: &str, indent: &str) -> String {
    // Fix: 冒号后缺空格  key:value → key: value，并对值部分应用流式修正
    if let Some(colon_pos) = content_part.find(':') {
        if colon_pos > 0 {
            let after_colon = &content_part[colon_pos + 1..];
            if !content_part.starts_with('{')
                && !content_part.starts_with('[')
                && !after_colon.starts_with("//")
            {
                let key_part = &content_part[..colon_pos];
                let val_part = after_colon.trim_start();
                let fixed_val = fix_flow_value(val_part);
                // 冒号后缺空格时补上，已有空格时也处理值部分
                if after_colon.starts_with(|c: char| {
                    c.is_alphanumeric() || c == '"' || c == '\'' || c == '[' || c == '{' || c == '-'
                }) {
                    return format!("{}{}: {}", indent, key_part.trim(), fixed_val);
                } else if after_colon.starts_with(' ') && fixed_val != val_part {
                    // 已有空格但值部分被流式修正了
                    return format!("{}{}: {}", indent, key_part.trim(), fixed_val);
                }
            }
        }
    }

    // Fix: 流式列表 [1,2,3] → [1, 2, 3]
    if content_part.starts_with('[') && content_part.contains(',') {
        if let Some(fixed) = fix_flow_sequence(content_part) {
            return format!("{}{}", indent, fixed);
        }
    }

    // Fix: 流式映射 {a:1} → {a: 1}
    if content_part.starts_with('{') && content_part.contains(':') {
        if let Some(fixed) = fix_flow_mapping(content_part) {
            return format!("{}{}", indent, fixed);
        }
    }

    format!("{}{}", indent, content_part)
}

/// 对值部分应用流式格式修正（用于 key: [1,2,3] 和 key: {a:1} 场景）
fn fix_flow_value(val: &str) -> String {
    if val.starts_with('[') && val.contains(',') {
        if let Some(fixed) = fix_flow_sequence(val) {
            return fixed;
        }
    }
    if val.starts_with('{') && val.contains(':') {
        if let Some(fixed) = fix_flow_mapping(val) {
            return fixed;
        }
    }
    val.to_string()
}

/// 修正流式列表中的空格: [1,2,3] → [1, 2, 3]
fn fix_flow_sequence(s: &str) -> Option<String> {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    if len < 2 || chars[0] != '[' || chars[len - 1] != ']' {
        return None;
    }
    let mut result = String::from("[");
    let mut in_string = false;
    let mut string_char = '"';
    let mut depth = 0;
    let mut i = 1;

    while i < len - 1 {
        let ch = chars[i];
        if in_string {
            if ch == '\\' && i + 1 < len - 1 {
                result.push(ch);
                result.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if ch == string_char {
                in_string = false;
            }
            result.push(ch);
            i += 1;
            continue;
        }
        match ch {
            '"' | '\'' => {
                in_string = true;
                string_char = ch;
                result.push(ch);
            }
            '[' => {
                depth += 1;
                result.push(ch);
            }
            ']' => {
                depth -= 1;
                result.push(ch);
            }
            ',' if depth == 0 => {
                result.push(',');
                if i + 1 < len - 1 && chars[i + 1] != ' ' {
                    result.push(' ');
                }
            }
            _ => result.push(ch),
        }
        i += 1;
    }
    result.push(']');
    Some(result)
}

/// 修正流式映射中的空格: {a:1, b:2} → {a: 1, b: 2}
fn fix_flow_mapping(s: &str) -> Option<String> {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    if len < 2 || chars[0] != '{' || chars[len - 1] != '}' {
        return None;
    }
    let mut result = String::from("{");
    let mut in_string = false;
    let mut string_char = '"';
    let mut depth = 0;
    let mut i = 1;

    while i < len - 1 {
        let ch = chars[i];
        if in_string {
            if ch == '\\' && i + 1 < len - 1 {
                result.push(ch);
                result.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if ch == string_char {
                in_string = false;
            }
            result.push(ch);
            i += 1;
            continue;
        }
        match ch {
            '"' | '\'' => {
                in_string = true;
                string_char = ch;
                result.push(ch);
            }
            '{' => {
                depth += 1;
                result.push(ch);
            }
            '}' => {
                depth -= 1;
                result.push(ch);
            }
            ':' if depth == 0 => {
                result.push(':');
                if i + 1 < len - 1 && chars[i + 1] != ' ' {
                    result.push(' ');
                }
            }
            ',' if depth == 0 => {
                result.push(',');
                if i + 1 < len - 1 && chars[i + 1] != ' ' {
                    result.push(' ');
                }
            }
            _ => result.push(ch),
        }
        i += 1;
    }
    result.push('}');
    Some(result)
}

/// 将前导 Tab 替换为对齐到指定宽度的空格
/// 例如 tab_width=2 时，`\tX` → `  X`，` \tX` → `  X`，`  \tX` → `    X`
fn expand_tabs_to_indent(content: &str, tab_width: usize) -> String {
    content
        .lines()
        .map(|line| {
            let mut col = 0;
            let mut leading = String::new();
            let mut content_start = line.len();
            for (i, ch) in line.char_indices() {
                if ch == '\t' {
                    let spaces = tab_width - (col % tab_width);
                    leading.push_str(&" ".repeat(spaces));
                    col += spaces;
                } else if ch == ' ' {
                    leading.push(ch);
                    col += 1;
                } else {
                    content_start = i;
                    break;
                }
            }
            format!("{}{}", leading, &line[content_start..])
        })
        .collect::<Vec<_>>()
        .join("\n")
}
