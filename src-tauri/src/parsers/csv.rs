use crate::FormatError;

pub fn check(content: &str) -> (Vec<FormatError>, Option<String>) {
    let content = content.trim_start_matches('\u{feff}');
    let mut errors = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        errors.push(FormatError {
            line: None,
            col: None,
            near: None,
            raw: "空文件".into(),
            friendly: "CSV 文件内容为空".into(),
        });
        return (errors, None);
    }

    let header_count = count_fields(lines[0]);

    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let count = count_fields(line);
        if count != header_count {
            let near = if line.len() > 40 {
                line.chars().take(40).collect::<String>().trim().to_string()
            } else {
                line.trim().to_string()
            };
            errors.push(FormatError {
                line: Some((i + 1) as u32),
                col: None,
                near: Some(near),
                raw: format!("列数不一致: 期望 {} 列，实际 {} 列", header_count, count),
                friendly: format!(
                    "第 {} 行有 {} 列，但表头有 {} 列，列数不一致",
                    i + 1,
                    count,
                    header_count
                ),
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

/// 计算字段数量（正确处理引号内的逗号）
fn count_fields(line: &str) -> usize {
    let mut count = 0;
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                if in_quotes {
                    if chars.peek() == Some(&'"') {
                        chars.next();
                    } else {
                        in_quotes = false;
                    }
                } else {
                    in_quotes = true;
                }
            }
            ',' if !in_quotes => {
                count += 1;
            }
            _ => {}
        }
    }
    count + 1
}

/// 全面的 CSV 修正，覆盖以下错误类型：
/// 1. 列数不齐 → 补齐或截断
/// 2. BOM 移除（UTF-8 BOM: EF BB BF）
/// 3. 行尾符统一（CRLF → LF）
/// 4. 未闭合引号补全
/// 5. 末尾多余逗号移除
/// 6. 空行移除
/// 7. 错误分隔符检测（tab/分号/管道符 → 逗号，尊重引号内容）
pub fn simple_fix(content: &str) -> String {
    // Pass 1: 移除 BOM
    let pass1 = content.trim_start_matches('\u{feff}');

    // Pass 2: CRLF → LF
    let pass2 = pass1.replace("\r\n", "\n").replace('\r', "\n");

    // Pass 3: 修正未闭合引号
    let pass3 = fix_unclosed_quotes_csv(&pass2);

    // Pass 4: 移除空行
    let pass4 = remove_empty_lines(&pass3);

    // Pass 5: 检测并修正错误分隔符（尊重引号内容）
    let pass5 = fix_wrong_delimiter(&pass4);

    // Pass 6: 列数对齐（自动处理多余/缺少的逗号）
    align_columns(&pass5)
}

/// 修正未闭合的引号
fn fix_unclosed_quotes_csv(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();

    for line in &lines {
        if line.trim().is_empty() {
            result.push(line.to_string());
            continue;
        }

        let mut fixed = String::with_capacity(line.len());
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '"' => {
                    if in_quotes {
                        if chars.peek() == Some(&'"') {
                            fixed.push('"');
                            chars.next();
                            fixed.push('"');
                        } else {
                            in_quotes = false;
                            fixed.push('"');
                        }
                    } else {
                        in_quotes = true;
                        fixed.push('"');
                    }
                }
                _ => fixed.push(ch),
            }
        }

        if in_quotes {
            fixed.push('"');
        }

        result.push(fixed);
    }

    result.join("\n")
}

/// 移除空行
fn remove_empty_lines(content: &str) -> String {
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect::<Vec<&str>>()
        .join("\n")
}

/// 检测并修正错误分隔符（尊重引号内的内容）
fn fix_wrong_delimiter(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return content.to_string();
    }

    // 统计各种分隔符在引号外出现的次数
    let mut tab_count = 0;
    let mut semicolon_count = 0;
    let mut pipe_count = 0;
    let mut comma_count = 0;

    for line in &lines {
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();
        while let Some(ch) = chars.next() {
            match ch {
                '"' => {
                    if in_quotes && chars.peek() == Some(&'"') {
                        chars.next();
                    } else {
                        in_quotes = !in_quotes;
                    }
                }
                '\t' if !in_quotes => tab_count += 1,
                ';' if !in_quotes => semicolon_count += 1,
                '|' if !in_quotes => pipe_count += 1,
                ',' if !in_quotes => comma_count += 1,
                _ => {}
            }
        }
    }

    // 如果逗号已经是主要分隔符，不替换
    if comma_count >= tab_count && comma_count >= semicolon_count && comma_count >= pipe_count {
        return content.to_string();
    }

    // 确定要替换的分隔符
    let target = if tab_count > comma_count && tab_count > semicolon_count && tab_count > pipe_count
    {
        '\t'
    } else if semicolon_count > comma_count && semicolon_count > pipe_count {
        ';'
    } else if pipe_count > comma_count {
        '|'
    } else {
        return content.to_string();
    };

    // 替换引号外的目标分隔符为逗号
    let mut result = String::with_capacity(content.len());
    let mut in_quotes = false;
    let mut chars = content.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                if in_quotes && chars.peek() == Some(&'"') {
                    result.push('"');
                    chars.next();
                    result.push('"');
                } else {
                    in_quotes = !in_quotes;
                    result.push('"');
                }
            }
            c if c == target && !in_quotes => {
                result.push(',');
            }
            c => result.push(c),
        }
    }
    result
}

/// 列数对齐（补齐或截断到表头列数）
fn align_columns(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return content.to_string();
    }

    let header_count = count_fields(lines[0]);
    let mut result = Vec::new();

    for line in &lines {
        if line.trim().is_empty() {
            continue;
        }
        let count = count_fields(line);
        if count < header_count {
            let mut fixed = line.to_string();
            for _ in 0..(header_count - count) {
                fixed.push(',');
            }
            result.push(fixed);
        } else if count > header_count {
            let mut in_quotes = false;
            let mut field_count = 0;
            let mut chars_iter = line.chars().peekable();
            let mut fixed = String::new();
            while let Some(ch) = chars_iter.next() {
                match ch {
                    '"' => {
                        if in_quotes && chars_iter.peek() == Some(&'"') {
                            fixed.push(ch);
                            if let Some(escaped) = chars_iter.next() {
                                fixed.push(escaped);
                            }
                            continue;
                        } else {
                            in_quotes = !in_quotes;
                        }
                    }
                    ',' if !in_quotes => {
                        field_count += 1;
                        if field_count >= header_count {
                            break;
                        }
                    }
                    _ => {}
                }
                fixed.push(ch);
            }
            result.push(fixed);
        } else {
            result.push(line.to_string());
        }
    }

    result.join("\n")
}
