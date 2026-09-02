use crate::FormatError;

pub fn check(content: &str) -> (Vec<FormatError>, Option<String>) {
    let content = content.trim_start_matches('\u{feff}');
    let mut errors = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }

        if let Some(eq_pos) = trimmed.find('=') {
            let raw_key = trimmed[..eq_pos].trim();
            let (_is_export, key) = if let Some(rest) = raw_key.strip_prefix("export") {
                if rest.starts_with(char::is_whitespace) {
                    (true, rest.trim())
                } else {
                    (false, raw_key)
                }
            } else {
                (false, raw_key)
            };

            if key.is_empty() {
                errors.push(FormatError {
                    line: Some((i + 1) as u32),
                    col: Some(1),
                    near: Some(trimmed.to_string()),
                    raw: "空的键名".into(),
                    friendly: format!("第 {} 行等号左边没有键名", i + 1),
                });
                continue;
            }

            let invalid_chars: Vec<char> = key
                .chars()
                .filter(|c| !c.is_alphanumeric() && *c != '_' && *c != '-')
                .collect();
            if !invalid_chars.is_empty() {
                errors.push(FormatError {
                    line: Some((i + 1) as u32),
                    col: None,
                    near: Some(key.to_string()),
                    raw: format!("键名包含非法字符: {:?}", invalid_chars),
                    friendly: format!(
                        "第 {} 行键名 \"{}\" 包含非法字符 {}，环境变量名只能包含字母、数字和下划线",
                        i + 1,
                        key,
                        invalid_chars.iter().collect::<String>()
                    ),
                });
            }

            let value = trimmed[eq_pos + 1..].trim();
            // 检查引号是否正确闭合（统计未转义的引号数量）
            if value.starts_with('"') {
                let count = count_unescaped_quotes(value, '"');
                if !count.is_multiple_of(2) {
                    errors.push(FormatError {
                        line: Some((i + 1) as u32),
                        col: Some((eq_pos + 1) as u32),
                        near: Some(value.to_string()),
                        raw: "未闭合的引号".into(),
                        friendly: format!("第 {} 行值的双引号没有闭合", i + 1),
                    });
                }
            } else if value.starts_with('\'') {
                let count = count_unescaped_quotes(value, '\'');
                if !count.is_multiple_of(2) {
                    errors.push(FormatError {
                        line: Some((i + 1) as u32),
                        col: Some((eq_pos + 1) as u32),
                        near: Some(value.to_string()),
                        raw: "未闭合的引号".into(),
                        friendly: format!("第 {} 行值的单引号没有闭合", i + 1),
                    });
                }
            }
        } else {
            errors.push(FormatError {
                line: Some((i + 1) as u32),
                col: None,
                near: Some(trimmed.to_string()),
                raw: "缺少等号".into(),
                friendly: format!("第 {} 行不是有效的环境变量格式，缺少 = 号", i + 1),
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

/// 统计未转义的引号数量
fn count_unescaped_quotes(s: &str, quote: char) -> usize {
    let mut count = 0;
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' && chars.peek() == Some(&quote) {
            chars.next(); // 跳过转义的引号
        } else if ch == quote {
            count += 1;
        }
    }
    count
}

/// 全面的 ENV 修正，覆盖以下错误类型：
/// 1. 缺失等号 → 添加 = 和空值
/// 2. 未闭合引号补全
/// 3. 键名非法字符移除
/// 4. 等号两侧空格规范化
/// 5. 值为空时添加空字符串
/// 6. BOM 移除
/// 7. 行尾符统一（CRLF → LF）
/// 8. 注释行统一（# 开头）
pub fn simple_fix(content: &str) -> String {
    // Pass 1: 移除 BOM
    let pass1 = content.trim_start_matches('\u{feff}');

    // Pass 2: CRLF → LF
    let pass2 = pass1.replace("\r\n", "\n").replace('\r', "\n");

    let lines: Vec<&str> = pass2.lines().collect();
    let mut result = Vec::new();

    for line in &lines {
        let trimmed = line.trim();

        // 空行保持不变
        if trimmed.is_empty() {
            result.push(String::new());
            continue;
        }

        // 注释行统一为 # 开头
        if let Some(rest) = trimmed.strip_prefix(';') {
            result.push(format!("#{}", rest));
            continue;
        }
        if trimmed.starts_with('#') {
            result.push(line.to_string());
            continue;
        }

        // 缺失等号 → 添加 = 和空值
        if !trimmed.contains('=') {
            let (is_export, raw_key) = if let Some(rest) = trimmed.strip_prefix("export") {
                if rest.starts_with(char::is_whitespace) {
                    (true, rest.trim())
                } else {
                    (false, trimmed)
                }
            } else {
                (false, trimmed)
            };
            let key: String = raw_key
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect();
            if !key.is_empty() {
                if is_export {
                    result.push(format!("export {}=", key));
                } else {
                    result.push(format!("{}=", key));
                }
            }
            continue;
        }

        if let Some(eq_pos) = trimmed.find('=') {
            let raw_prefix = trimmed[..eq_pos].trim();
            let raw_value = &trimmed[eq_pos + 1..];

            let (is_export, raw_key) = if let Some(rest) = raw_prefix.strip_prefix("export") {
                if rest.starts_with(char::is_whitespace) {
                    (true, rest.trim())
                } else {
                    (false, raw_prefix)
                }
            } else {
                (false, raw_prefix)
            };

            // 修正键名：移除非法字符
            let key: String = raw_key
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect();

            if key.is_empty() {
                continue;
            }

            // 规范化等号两侧空格
            let value = raw_value.trim();

            // 修正未闭合引号
            let fixed_value = fix_unclosed_quotes_env(value);

            if is_export {
                result.push(format!("export {}={}", key, fixed_value));
            } else {
                result.push(format!("{}={}", key, fixed_value));
            }
            continue;
        }

        result.push(line.to_string());
    }

    result.join("\n")
}

/// 修正未闭合的引号
fn fix_unclosed_quotes_env(value: &str) -> String {
    if value.is_empty() {
        return value.to_string();
    }

    // 双引号字符串
    if value.starts_with('"') {
        let count = count_unescaped_quotes(value, '"');
        if count.is_multiple_of(2) {
            return value.to_string();
        }
        return format!("{}\"", value);
    }

    // 单引号字符串
    if value.starts_with('\'') {
        let count = count_unescaped_quotes(value, '\'');
        if count.is_multiple_of(2) {
            return value.to_string();
        }
        return format!("{}'", value);
    }

    value.to_string()
}
