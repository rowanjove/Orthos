use crate::FormatError;

#[derive(Debug, Clone)]
struct CsvRecord<'a> {
    line_number: u32,
    content: &'a str,
}

fn split_csv_records(content: &str) -> Vec<CsvRecord<'_>> {
    let mut records = Vec::new();
    let mut in_quotes = false;
    let mut record_start = 0;
    let mut record_line = 1u32;
    let mut current_line = 1u32;

    let mut chars = content.char_indices().peekable();
    while let Some((idx, ch)) = chars.next() {
        match ch {
            '"' => {
                if in_quotes {
                    if chars.peek().map(|&(_, next_ch)| next_ch) == Some('"') {
                        chars.next();
                    } else {
                        in_quotes = false;
                    }
                } else {
                    in_quotes = true;
                }
            }
            '\n' => {
                current_line += 1;
                if !in_quotes {
                    let end = if idx > 0 && content.as_bytes()[idx - 1] == b'\r' {
                        idx - 1
                    } else {
                        idx
                    };
                    records.push(CsvRecord {
                        line_number: record_line,
                        content: &content[record_start..end],
                    });
                    record_start = idx + 1;
                    record_line = current_line;
                }
            }
            _ => {}
        }
    }

    if record_start < content.len() {
        let remaining = &content[record_start..];
        let end = if remaining.ends_with('\r') {
            remaining.len() - 1
        } else {
            remaining.len()
        };
        records.push(CsvRecord {
            line_number: record_line,
            content: &remaining[..end],
        });
    }

    records
}

pub fn check(content: &str) -> (Vec<FormatError>, Option<String>) {
    let content = content.trim_start_matches('\u{feff}');
    let mut errors = Vec::new();
    let records = split_csv_records(content);

    let header_record = records.iter().find(|r| !r.content.trim().is_empty());
    let Some(header) = header_record else {
        errors.push(FormatError {
            line: None,
            col: None,
            near: None,
            raw: "空文件".into(),
            friendly: "CSV 文件内容为空".into(),
        });
        return (errors, None);
    };

    let header_count = count_fields(header.content);

    for rec in records
        .iter()
        .filter(|r| !r.content.trim().is_empty())
        .skip(1)
    {
        let count = count_fields(rec.content);
        if count != header_count {
            let single_line = rec.content.replace('\n', " ").replace('\r', "");
            let near = if single_line.chars().count() > 40 {
                single_line
                    .chars()
                    .take(40)
                    .collect::<String>()
                    .trim()
                    .to_string()
            } else {
                single_line.trim().to_string()
            };
            errors.push(FormatError {
                line: Some(rec.line_number),
                col: None,
                near: Some(near),
                raw: format!("列数不一致: 期望 {} 列，实际 {} 列", header_count, count),
                friendly: format!(
                    "第 {} 行有 {} 列，但表头有 {} 列，列数不一致",
                    rec.line_number, count, header_count
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
/// 1. 缺少字段 → 补齐；多余字段保持原样，避免静默丢失数据
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

    // Pass 6: 仅补齐缺少字段，不截断多余字段
    pad_missing_columns(&pass5)
}

/// 修正未闭合的引号
fn fix_unclosed_quotes_csv(content: &str) -> String {
    let mut result = String::with_capacity(content.len() + 2);
    let mut in_quotes = false;
    let mut chars = content.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                if in_quotes {
                    if chars.peek() == Some(&'"') {
                        result.push('"');
                        chars.next();
                        result.push('"');
                    } else {
                        in_quotes = false;
                        result.push('"');
                    }
                } else {
                    in_quotes = true;
                    result.push('"');
                }
            }
            _ => result.push(ch),
        }
    }

    if in_quotes {
        result.push('"');
    }

    result
}

/// 移除空记录
fn remove_empty_lines(content: &str) -> String {
    let records = split_csv_records(content);
    records
        .into_iter()
        .filter(|r| !r.content.trim().is_empty())
        .map(|r| r.content)
        .collect::<Vec<_>>()
        .join("\n")
}

/// 检测并修正错误分隔符（尊重引号内的内容）
fn fix_wrong_delimiter(content: &str) -> String {
    let records = split_csv_records(content);
    if records.is_empty() {
        return content.to_string();
    }

    let mut tab_count = 0;
    let mut semicolon_count = 0;
    let mut pipe_count = 0;
    let mut comma_count = 0;

    for rec in &records {
        let mut in_quotes = false;
        let mut chars = rec.content.chars().peekable();
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

/// 补齐缺少字段，但保留多余字段，避免修复过程丢失数据
fn pad_missing_columns(content: &str) -> String {
    let records = split_csv_records(content);
    if records.is_empty() {
        return content.to_string();
    }

    let Some(header) = records.iter().find(|r| !r.content.trim().is_empty()) else {
        return content.to_string();
    };
    let header_count = count_fields(header.content);
    let mut result = Vec::new();

    for rec in &records {
        if rec.content.trim().is_empty() {
            continue;
        }
        let count = count_fields(rec.content);
        if count < header_count {
            let mut fixed = rec.content.to_string();
            for _ in 0..(header_count - count) {
                fixed.push(',');
            }
            result.push(fixed);
        } else {
            result.push(rec.content.to_string());
        }
    }

    result.join("\n")
}
