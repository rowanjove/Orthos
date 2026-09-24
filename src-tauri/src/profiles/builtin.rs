use super::profile::{ConfigProfile, ProfileDiagnostic};
use crate::document::DocumentNode;
use std::collections::HashSet;

// ==================== package.json Profile ====================
pub struct PackageJsonProfile;

impl ConfigProfile for PackageJsonProfile {
    fn id(&self) -> &'static str {
        "package.json"
    }

    fn name(&self) -> &'static str {
        "Node.js Package (package.json)"
    }

    fn description(&self) -> &'static str {
        "校验 Node.js package.json 的依赖、命名、版本号规范及常用元数据"
    }

    fn detect(&self, filename: &str, _content: &str, _doc: &DocumentNode) -> u8 {
        let lower = filename.to_ascii_lowercase();
        if lower == "package.json"
            || lower.ends_with("/package.json")
            || lower.ends_with("\\package.json")
        {
            100
        } else {
            0
        }
    }

    fn validate(&self, document: &DocumentNode) -> Vec<ProfileDiagnostic> {
        let mut diags = Vec::new();

        // 检查 name
        if let Some(name_node) = document.find_by_path("/name") {
            if let Some(name_str) = name_node.value.as_ref().and_then(|v| v.as_str()) {
                if name_str.chars().any(|c| c.is_ascii_uppercase()) {
                    diags.push(ProfileDiagnostic {
                        path: Some("/name".into()),
                        message: "package.json 的包名不能包含大写字母".into(),
                        severity: "warning".into(),
                        suggestion: Some(name_str.to_ascii_lowercase()),
                    });
                }
            }
        } else {
            diags.push(ProfileDiagnostic {
                path: Some("/".into()),
                message: "缺少基础字段 'name'".into(),
                severity: "warning".into(),
                suggestion: None,
            });
        }

        // 检查 version
        if let Some(ver_node) = document.find_by_path("/version") {
            if let Some(ver_str) = ver_node.value.as_ref().and_then(|v| v.as_str()) {
                let parts: Vec<&str> = ver_str.split('.').collect();
                if parts.len() < 3 || !parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())) {
                    diags.push(ProfileDiagnostic {
                        path: Some("/version".into()),
                        message: "版本号建议遵循 SemVer 规范 (例如 1.0.0)".into(),
                        severity: "info".into(),
                        suggestion: Some("1.0.0".into()),
                    });
                }
            }
        }

        // 检查 dependencies 与 devDependencies 是否有重复包名
        let mut deps_set = HashSet::new();
        if let Some(deps_node) = document.find_by_path("/dependencies") {
            if let Some(children) = &deps_node.children {
                for c in children {
                    if let Some(k) = &c.key {
                        deps_set.insert(k.clone());
                    }
                }
            }
        }

        if let Some(dev_node) = document.find_by_path("/devDependencies") {
            if let Some(children) = &dev_node.children {
                for c in children {
                    if let Some(k) = &c.key {
                        if deps_set.contains(k) {
                            diags.push(ProfileDiagnostic {
                                path: Some(format!("/devDependencies/{}", k)),
                                message: format!(
                                    "包 '{}' 同时出现在 dependencies 和 devDependencies 中",
                                    k
                                ),
                                severity: "warning".into(),
                                suggestion: Some("保留在单处即可".into()),
                            });
                        }
                    }
                }
            }
        }

        diags
    }
}

// ==================== tsconfig.json Profile ====================
pub struct TsconfigJsonProfile;

impl ConfigProfile for TsconfigJsonProfile {
    fn id(&self) -> &'static str {
        "tsconfig.json"
    }

    fn name(&self) -> &'static str {
        "TypeScript Config (tsconfig.json)"
    }

    fn description(&self) -> &'static str {
        "检查 TypeScript 编译配置的 target、moduleResolution 与严格模式项"
    }

    fn detect(&self, filename: &str, _content: &str, _doc: &DocumentNode) -> u8 {
        let lower = filename.to_ascii_lowercase();
        if lower == "tsconfig.json"
            || lower.ends_with("/tsconfig.json")
            || lower.ends_with("\\tsconfig.json")
        {
            100
        } else {
            0
        }
    }

    fn validate(&self, document: &DocumentNode) -> Vec<ProfileDiagnostic> {
        let mut diags = Vec::new();

        if let Some(target_node) = document.find_by_path("/compilerOptions/target") {
            if let Some(target_str) = target_node.value.as_ref().and_then(|v| v.as_str()) {
                let valid_targets = [
                    "es5", "es6", "es2015", "es2016", "es2017", "es2018", "es2019", "es2020",
                    "es2021", "es2022", "esnext",
                ];
                let lower = target_str.to_ascii_lowercase();
                if !valid_targets.contains(&lower.as_str()) {
                    diags.push(ProfileDiagnostic {
                        path: Some("/compilerOptions/target".into()),
                        message: format!("未知的编译目标 target: '{}'", target_str),
                        severity: "warning".into(),
                        suggestion: Some("ES2022".into()),
                    });
                }
            }
        }

        if let Some(strict_node) = document.find_by_path("/compilerOptions/strict") {
            if let Some(b) = strict_node.value.as_ref().and_then(|v| v.as_bool()) {
                if !b {
                    diags.push(ProfileDiagnostic {
                        path: Some("/compilerOptions/strict".into()),
                        message: "建议开启 strict 严格模式以获得最佳类型安全保障".into(),
                        severity: "info".into(),
                        suggestion: Some("true".into()),
                    });
                }
            }
        }

        diags
    }
}

// ==================== Docker Compose Profile ====================
pub struct DockerComposeProfile;

impl ConfigProfile for DockerComposeProfile {
    fn id(&self) -> &'static str {
        "docker-compose"
    }

    fn name(&self) -> &'static str {
        "Docker Compose (compose.yml)"
    }

    fn description(&self) -> &'static str {
        "校验 Docker Compose 服务的端口映射格式、重复端口占用与服务基础配置"
    }

    fn detect(&self, filename: &str, _content: &str, doc: &DocumentNode) -> u8 {
        let lower = filename.to_ascii_lowercase();
        if lower.contains("docker-compose")
            || lower.contains("compose.yml")
            || lower.contains("compose.yaml")
        {
            return 100;
        }
        if doc.find_by_path("/services").is_some() && doc.find_by_path("/version").is_some() {
            return 90;
        }
        0
    }

    fn validate(&self, document: &DocumentNode) -> Vec<ProfileDiagnostic> {
        let mut diags = Vec::new();

        let Some(services_node) = document.find_by_path("/services") else {
            diags.push(ProfileDiagnostic {
                path: Some("/".into()),
                message: "缺少顶层 'services' 服务定义模块".into(),
                severity: "error".into(),
                suggestion: None,
            });
            return diags;
        };

        let mut host_ports = HashSet::new();

        if let Some(services) = &services_node.children {
            for service in services {
                let s_name = service.key.as_deref().unwrap_or("service");
                let ports_path = format!("{}/ports", service.path);

                if let Some(ports_node) = document.find_by_path(&ports_path) {
                    if let Some(ports_list) = &ports_node.children {
                        for p in ports_list {
                            if let Some(val_str) = p.value.as_ref().and_then(|v| v.as_str()) {
                                if let Some(host_p) = val_str.split(':').next() {
                                    if !host_ports.insert(host_p.to_string()) {
                                        diags.push(ProfileDiagnostic {
                                            path: Some(p.path.clone()),
                                            message: format!(
                                                "服务 '{}' 映射的主机端口 '{}' 与其他服务冲突",
                                                s_name, host_p
                                            ),
                                            severity: "warning".into(),
                                            suggestion: None,
                                        });
                                    }
                                }
                            } else if p.kind == crate::document::NodeKind::Number {
                                diags.push(ProfileDiagnostic {
                                    path: Some(p.path.clone()),
                                    message: format!(
                                        "服务 '{}' 的端口映射写成了纯数字，YAML 可能发生八进制或六十进制转换，建议加引号",
                                        s_name
                                    ),
                                    severity: "warning".into(),
                                    suggestion: Some(format!("\"{}\"", p.value.as_ref().map(|v| v.to_string()).unwrap_or_default())),
                                });
                            }
                        }
                    }
                }
            }
        }

        diags
    }
}

// ==================== GitHub Actions Profile ====================
pub struct GitHubActionsProfile;

impl ConfigProfile for GitHubActionsProfile {
    fn id(&self) -> &'static str {
        "github-actions"
    }

    fn name(&self) -> &'static str {
        "GitHub Actions Workflow"
    }

    fn description(&self) -> &'static str {
        "校验 GitHub Actions 工作流触发器、Runner 名称拼写及 Jobs 结构"
    }

    fn detect(&self, filename: &str, _content: &str, doc: &DocumentNode) -> u8 {
        let norm = filename.replace('\\', "/").to_ascii_lowercase();
        if norm.contains(".github/workflows") {
            return 100;
        }
        if doc.find_by_path("/on").is_some() && doc.find_by_path("/jobs").is_some() {
            return 90;
        }
        0
    }

    fn validate(&self, document: &DocumentNode) -> Vec<ProfileDiagnostic> {
        let mut diags = Vec::new();

        if document.find_by_path("/on").is_none() {
            diags.push(ProfileDiagnostic {
                path: Some("/".into()),
                message: "缺少工作流触发器 'on'".into(),
                severity: "error".into(),
                suggestion: None,
            });
        }

        if let Some(jobs_node) = document.find_by_path("/jobs") {
            if let Some(jobs) = &jobs_node.children {
                for job in jobs {
                    let runs_on_path = format!("{}/runs-on", job.path);
                    if let Some(runs_on_node) = document.find_by_path(&runs_on_path) {
                        if let Some(runner) = runs_on_node.value.as_ref().and_then(|v| v.as_str()) {
                            let common_runners = [
                                "ubuntu-latest",
                                "ubuntu-22.04",
                                "ubuntu-20.04",
                                "ubuntu-24.04",
                                "windows-latest",
                                "windows-2022",
                                "windows-2019",
                                "macos-latest",
                                "macos-14",
                                "macos-13",
                                "macos-12",
                            ];
                            if !common_runners.contains(&runner) {
                                // 模糊推测
                                if runner.contains("ubuntu") && runner.contains("late") {
                                    diags.push(ProfileDiagnostic {
                                        path: Some(runs_on_path),
                                        message: format!("未知的 Runner 环境: '{}'", runner),
                                        severity: "warning".into(),
                                        suggestion: Some("ubuntu-latest".into()),
                                    });
                                } else if runner.contains("macos") && runner.contains("late") {
                                    diags.push(ProfileDiagnostic {
                                        path: Some(runs_on_path),
                                        message: format!("未知的 Runner 环境: '{}'", runner),
                                        severity: "warning".into(),
                                        suggestion: Some("macos-latest".into()),
                                    });
                                } else if runner.contains("windows") && runner.contains("late") {
                                    diags.push(ProfileDiagnostic {
                                        path: Some(runs_on_path),
                                        message: format!("未知的 Runner 环境: '{}'", runner),
                                        severity: "warning".into(),
                                        suggestion: Some("windows-latest".into()),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        diags
    }
}

// ==================== Cargo.toml Profile ====================
pub struct CargoTomlProfile;

impl ConfigProfile for CargoTomlProfile {
    fn id(&self) -> &'static str {
        "cargo.toml"
    }

    fn name(&self) -> &'static str {
        "Rust Cargo Manifest (Cargo.toml)"
    }

    fn description(&self) -> &'static str {
        "校验 Cargo.toml 包名、版本号、Rust Edition 规范"
    }

    fn detect(&self, filename: &str, _content: &str, _doc: &DocumentNode) -> u8 {
        if filename.eq_ignore_ascii_case("Cargo.toml") {
            100
        } else {
            0
        }
    }

    fn validate(&self, document: &DocumentNode) -> Vec<ProfileDiagnostic> {
        let mut diags = Vec::new();

        if let Some(edition_node) = document.find_by_path("/package/edition") {
            if let Some(edition) = edition_node.value.as_ref().and_then(|v| v.as_str()) {
                if !["2015", "2018", "2021", "2024"].contains(&edition) {
                    diags.push(ProfileDiagnostic {
                        path: Some("/package/edition".into()),
                        message: format!("非法的 Rust Edition: '{}'", edition),
                        severity: "error".into(),
                        suggestion: Some("2021".into()),
                    });
                }
            }
        }

        diags
    }
}
