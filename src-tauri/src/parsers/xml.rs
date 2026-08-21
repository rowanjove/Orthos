use crate::FormatError;
use quick_xml::events::Event;
use quick_xml::Reader;

pub fn check(content: &str) -> (Vec<FormatError>, Option<String>) {
    let content = content.trim_start_matches('\u{feff}');
    let mut errors = Vec::new();
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => {
                let pos = reader.buffer_position() as usize;
                let (line, col) = pos_to_line_col(content, pos);
                let near = content
                    .lines()
                    .nth(line.saturating_sub(1) as usize)
                    .map(|l| preview_around_col(l, col));
                let raw_msg = e.to_string();
                errors.push(FormatError {
                    line: Some(line),
                    col: Some(col),
                    near,
                    friendly: friendly_message(&raw_msg),
                    raw: raw_msg,
                });
                break;
            }
        }
        buf.clear();
    }

    let corrected = if errors.is_empty() {
        None
    } else {
        let fixed = simple_fix(content);
        (fixed != content).then_some(fixed)
    };

    (errors, corrected)
}

fn preview_around_col(line: &str, col: u32) -> String {
    let col_index = col.saturating_sub(1) as usize;
    line.chars()
        .skip(col_index.saturating_sub(10))
        .take(40)
        .collect::<String>()
        .trim()
        .to_string()
}

fn pos_to_line_col(content: &str, pos: usize) -> (u32, u32) {
    let mut line = 1u32;
    let mut col = 0u32;
    for (i, ch) in content.char_indices() {
        if i >= pos {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col + 1)
}

fn friendly_message(raw: &str) -> String {
    let lower = raw.to_lowercase();
    if lower.contains("unclosed") || lower.contains("unexpected end") {
        return "XML 标签未闭合".into();
    }
    if lower.contains("mismatched") {
        return "XML 开闭标签不匹配".into();
    }
    if lower.contains("invalid") && lower.contains("name") {
        return "XML 标签名不合法".into();
    }
    if lower.contains("expected") && lower.contains("'>'") {
        return "标签未闭合，缺少 > 符号".into();
    }
    if lower.contains("attribute") {
        return "属性值缺少引号或格式不正确".into();
    }
    if lower.contains("comment") {
        return "注释未闭合，缺少 -->".into();
    }
    if lower.contains("cdata") {
        return "CDATA 段未正确闭合".into();
    }
    format!("XML 格式错误: {}", raw)
}

pub fn simple_fix(content: &str) -> String {
    let content = content.trim_start_matches('\u{feff}');
    let mut result = content.to_string();
    result = fix_unclosed_comments(&result);
    result = fix_unclosed_cdata(&result);
    result = fix_unclosed_angle_brackets(&result);
    result = fix_unquoted_attributes(&result);
    repair_tag_stack(&result)
}

fn fix_unclosed_comments(content: &str) -> String {
    let opens = content.matches("<!--").count();
    let closes = content.matches("-->").count();
    if opens > closes {
        format!("{}-->", content)
    } else {
        content.to_string()
    }
}

fn fix_unclosed_cdata(content: &str) -> String {
    let opens = content.matches("<![CDATA[").count();
    let closes = content.matches("]]>").count();
    if opens > closes {
        format!("{}]]>", content)
    } else {
        content.to_string()
    }
}

fn fix_unclosed_angle_brackets(content: &str) -> String {
    content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('<')
                && !trimmed.contains('>')
                && !trimmed.starts_with("<!--")
                && !trimmed.starts_with("<![CDATA[")
            {
                format!("{}>", line)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn fix_unquoted_attributes(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let mut result = String::with_capacity(content.len());
    let mut i = 0;

    while i < chars.len() {
        if chars[i] != '<' || i + 1 >= chars.len() || chars[i + 1] == '/' || chars[i + 1] == '!' {
            result.push(chars[i]);
            i += 1;
            continue;
        }

        while i < chars.len() && chars[i] != '>' {
            // 检查自闭合标签 />
            if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '>' {
                break;
            }
            if chars[i] == '=' && i + 1 < chars.len() {
                result.push('=');
                i += 1;
                if i < chars.len() && chars[i].is_whitespace() {
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                }
                if i < chars.len() && chars[i] != '"' && chars[i] != '\'' {
                    result.push('"');
                    while i < chars.len()
                        && chars[i] != '>'
                        && !chars[i].is_whitespace()
                        && !(chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '>')
                    {
                        result.push(chars[i]);
                        i += 1;
                    }
                    result.push('"');
                    continue;
                }
                continue;
            }
            result.push(chars[i]);
            i += 1;
        }

        if i < chars.len() {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

fn repair_tag_stack(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let mut result = String::with_capacity(content.len());
    let mut stack: Vec<String> = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] != '<' || i + 1 >= chars.len() {
            result.push(chars[i]);
            i += 1;
            continue;
        }

        // 处理 <!...> 注释和 CDATA
        if chars[i + 1] == '!' {
            let start = i;
            i += 2;
            // 注释 <!-- ... -->
            if i + 1 < chars.len() && chars[i] == '-' && chars[i + 1] == '-' {
                i += 2;
                while i + 2 < chars.len()
                    && !(chars[i] == '-' && chars[i + 1] == '-' && chars[i + 2] == '>')
                {
                    i += 1;
                }
                if i + 2 < chars.len() {
                    i += 3;
                }
            }
            // CDATA <![CDATA[ ... ]]>
            else if i + 7 <= chars.len()
                && chars[i..i + 7].iter().collect::<String>() == "[CDATA["
            {
                i += 7;
                while i + 2 < chars.len() {
                    if chars[i] == ']' && chars[i + 1] == ']' && chars[i + 2] == '>' {
                        break;
                    }
                    i += 1;
                }
                if i + 2 < chars.len() {
                    i += 3;
                }
            }
            // 其他 <! 指令
            else {
                while i < chars.len() && chars[i] != '>' {
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                }
            }
            result.extend(chars[start..i].iter());
            continue;
        }

        // 处理 <?...?> 处理指令
        if chars[i + 1] == '?' {
            let start = i;
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '?' && chars[i + 1] == '>') {
                i += 1;
            }
            if i + 1 < chars.len() {
                i += 2;
            }
            result.extend(chars[start..i].iter());
            continue;
        }

        // 闭合标签 </name>
        if chars[i + 1] == '/' {
            i += 2;
            let mut name = String::new();
            while i < chars.len() && chars[i] != '>' && !chars[i].is_whitespace() {
                name.push(chars[i]);
                i += 1;
            }
            while i < chars.len() && chars[i] != '>' {
                i += 1;
            }
            if i < chars.len() {
                i += 1;
            }

            close_until(&mut result, &mut stack, &name);
        } else {
            // 开始标签 <name ...> 或 <name/>
            let start = i;
            i += 1;
            let mut name = String::new();
            while i < chars.len() && chars[i] != '>' && chars[i] != '/' && !chars[i].is_whitespace()
            {
                name.push(chars[i]);
                i += 1;
            }
            // 跳过属性部分，找到 > 或 />
            let mut self_closing = false;
            while i < chars.len() && chars[i] != '>' {
                if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '>' {
                    self_closing = true;
                    break;
                }
                i += 1;
            }
            let end = if i < chars.len() { i + 1 } else { i };
            result.extend(chars[start..end].iter());
            i = end;

            if !name.is_empty() && !self_closing && name != "xml" && name != "DOCTYPE" {
                stack.push(name);
            }
        }
    }

    for tag in stack.iter().rev() {
        result.push_str(&format!("</{}>", tag));
    }

    result
}

fn close_until(result: &mut String, stack: &mut Vec<String>, name: &str) {
    if let Some(pos) = stack.iter().rposition(|tag| tag == name) {
        for tag in stack[pos + 1..].iter().rev() {
            result.push_str(&format!("</{}>", tag));
        }
        stack.truncate(pos);
        result.push_str(&format!("</{}>", name));
    } else if let Some(tag) = stack.pop() {
        result.push_str(&format!("</{}>", tag));
    }
    // 栈空且找不到匹配的闭合标签时，不输出（原字符串已包含该闭合标签）
}
