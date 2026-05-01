use crate::FormatError;
use std::collections::HashSet;

pub fn check(content: &str) -> (Vec<FormatError>, Option<String>) {
    let content = content.trim_start_matches('\u{feff}');
    let mut errors = Vec::new();
    let mut seen_keys: HashSet<String> = HashSet::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with('[') {
            if let Some(end) = trimmed.find(']') {
                let section = trimmed[1..end].trim().to_string();
                if section.is_empty() {
                    errors.push(FormatError {
                        line: Some((i + 1) as u32),
                        col: Some(1),
                        near: Some(trimmed.to_string()),
                        raw: "空的 section 名".into(),
                        friendly: "section 名称不能为空".into(),
                    });
                }
                seen_keys.clear();
                let after = trimmed[end + 1..].trim();
                if !after.is_empty() && !after.starts_with(';') && !after.starts_with('#') {
                    errors.push(FormatError {
                        line: Some((i + 1) as u32),
                        col: Some((end + 1) as u32),
                        near: Some(after.to_string()),
                        raw: format!("section 头后有意外内容: {}", after),
                        friendly: format!("第 {} 行 section 头后有意外内容", i + 1),
                    });
                }
            } else {
                errors.push(FormatError {
                    line: Some((i + 1) as u32),
                    col: None,
                    near: Some(trimmed.to_string()),
                    raw: "未闭合的 section 头".into(),
                    friendly: format!("第 {} 行的 [ 没有对应的 ]", i + 1),
                });
            }
            continue;
        }

        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim().to_string();
            if key.is_empty() {
                errors.push(FormatError {
                    line: Some((i + 1) as u32),
                    col: Some(1),
                    near: Some(trimmed.to_string()),
                    raw: "空的键名".into(),
                    friendly: format!("第 {} 行等号左边没有键名", i + 1),
                });
            } else if seen_keys.contains(&key) {
                errors.push(FormatError {
                    line: Some((i + 1) as u32),
                    col: None,
                    near: Some(key.clone()),
                    raw: format!("重复的键名: {}", key),
                    friendly: format!("第 {} 行键名 \"{}\" 在当前 section 中已存在", i + 1, key),
                });
            } else {
                seen_keys.insert(key);
            }
        } else {
            errors.push(FormatError {
                line: Some((i + 1) as u32),
                col: None,
                near: Some(trimmed.to_string()),
                raw: "缺少等号".into(),
                friendly: format!("第 {} 行不是有效的键值对，缺少 = 号", i + 1),
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

/// 全面的 INI 修正，覆盖以下错误类型：
/// 1. 重复键名删除（保留第一个）
/// 2. 缺失 section 头的键值对 → 添加默认 [default] section
/// 3. 缺失等号  key value → key = value
/// 4. 等号两侧空格规范化  key=value → key = value
/// 5. 未闭合 section 头  [section → [section]
/// 6. 空 section 名修正
/// 7. 注释风格统一（# → ;）
pub fn simple_fix(content: &str) -> String {
    let content = content.trim_start_matches('\u{feff}');
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut seen_keys: HashSet<String> = HashSet::new();
    let mut current_section = String::new();
    let mut has_orphan_keys = false;

    // 第一遍：检测是否有孤儿键（没有 section 头的键值对）
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') {
            continue;
        }
        if trimmed.find('=').is_some() {
            // 在第一个 section 之前有键值对
            if current_section.is_empty() {
                has_orphan_keys = true;
            }
        }
    }

    // 如果有孤儿键，添加默认 section
    if has_orphan_keys {
        result.push("[default]".to_string());
    }

    for line in &lines {
        let trimmed = line.trim();

        // 空行保持不变
        if trimmed.is_empty() {
            result.push(String::new());
            continue;
        }

        // 注释行统一为 ; 开头
        if let Some(rest) = trimmed.strip_prefix('#') {
            result.push(format!(";{}", rest));
            continue;
        }
        if trimmed.starts_with(';') {
            result.push(line.to_string());
            continue;
        }

        // Section 头处理
        if trimmed.starts_with('[') {
            if let Some(end) = trimmed.find(']') {
                let section = trimmed[1..end].trim().to_string();
                if section.is_empty() {
                    // 空 section 名 → 跳过
                    continue;
                }
                current_section = section.clone();
                seen_keys.clear();

                // 检查 section 后是否有内容
                let after = trimmed[end + 1..].trim();
                if after.is_empty() || after.starts_with(';') || after.starts_with('#') {
                    result.push(format!("[{}]", section));
                } else {
                    // section 后有内容 → 分离
                    result.push(format!("[{}]", section));
                    // 将后面的内容作为键值对处理
                    if let Some(eq_pos) = after.find('=') {
                        let key = after[..eq_pos].trim().to_string();
                        let value = after[eq_pos + 1..].trim();
                        if !key.is_empty() {
                            let full_key = format!("{}:{}", current_section, key);
                            if !seen_keys.contains(&full_key) {
                                seen_keys.insert(full_key);
                                result.push(format!("{} = {}", key, value));
                            }
                        }
                    }
                }
            } else {
                // 未闭合的 section 头
                if let Some(rest) = trimmed.strip_prefix('[') {
                    let section_name = rest.trim();
                    if !section_name.is_empty() {
                        current_section = section_name.to_string();
                        seen_keys.clear();
                        result.push(format!("[{}]", section_name));
                    } else {
                        result.push("[default]".to_string());
                        current_section = "default".to_string();
                        seen_keys.clear();
                    }
                }
            }
            continue;
        }

        // 键值对处理
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim().to_string();
            let value = trimmed[eq_pos + 1..].trim();

            if key.is_empty() {
                continue; // 跳过空键名
            }

            // 去重
            let full_key = format!("{}:{}", current_section, key);
            if seen_keys.contains(&full_key) {
                continue; // 跳过重复键
            }
            seen_keys.insert(full_key);

            // 规范化等号两侧空格
            result.push(format!("{} = {}", key, value));
            continue;
        }

        // 没有等号的行 → 尝试修复
        let key: String = trimmed
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .collect();
        if !key.is_empty() && key.len() < trimmed.len() {
            let rest = trimmed[key.len()..].trim();
            let full_key = format!("{}:{}", current_section, key);
            if !seen_keys.contains(&full_key) {
                seen_keys.insert(full_key);
                result.push(format!("{} = {}", key, rest));
                continue;
            }
        }

        result.push(line.to_string());
    }

    result.join("\n")
}
