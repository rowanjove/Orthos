# Orthos — 离线配置文件检查与安全修复

[简体中文](README.md) | [English](README.en.md)

Orthos 是一款面向 Windows 的本地配置文件检查器。把文件拖入窗口，或直接粘贴文本，即可检查 JSON、YAML、TOML、XML、CSV、INI 和 ENV，查看错误位置、中文说明与修正前后 Diff。所有处理都在本机完成，修正结果只有在再次通过解析后才可保存，原文件不会被自动覆盖。

[下载 Windows 版](https://github.com/rowanjove/Orthos/releases/latest) · [更新记录](CHANGELOG.md) · [报告问题](https://github.com/rowanjove/Orthos/issues)

[![CI](https://github.com/rowanjove/Orthos/actions/workflows/ci.yml/badge.svg)](https://github.com/rowanjove/Orthos/actions/workflows/ci.yml)
[![GitHub Release](https://img.shields.io/github/v/release/rowanjove/Orthos)](https://github.com/rowanjove/Orthos/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## 下载与使用

当前版本为 **v1.1.0**，支持 Windows 10/11 x64：

- `Orthos_1.1.0_x64-setup.exe`：安装版。
- `Orthos_1.1.0_x64_portable.exe`：免安装版，下载后直接运行。
- `SHA256SUMS.txt`：下载文件的 SHA-256 校验值。

从 [Releases](https://github.com/rowanjove/Orthos/releases/latest) 下载后，拖入配置文件或粘贴文本。先阅读诊断；如果 Orthos 给出修正结果，请检查 Diff，再决定是否保存新文件。

## 核心能力

| 能力 | 说明 |
| --- | --- |
| 七种格式 | JSON、YAML、TOML、XML、CSV、INI、ENV |
| 自动识别 | 根据文件扩展名或内容识别，也可手动指定 |
| 精确诊断 | 显示行列位置、附近原文和中文说明 |
| 安全修复 | 常见语法修正、Diff 预览、修正后二次校验 |
| 批量处理 | 多文件检查与批量保存修正结果 |
| JSON Schema | 检查 JSON 结构，不联网加载外部引用 |

单文件最大 **10 MB**，批量文件合计最大 **20 MB**。

## 安全与能力边界

Orthos 检查语法和结构，不判断业务含义。合法的端口数字不代表它适合你的部署环境；能够解析的配置也不等于应用一定接受它。

- 重复键保留原文并提示手动处理，不猜测应保留哪个值。
- 不删除 CSV 多余字段来强行通过检查。
- 修正内容必须再次通过对应解析器，才会提供保存入口。
- 原文件不会自动覆盖；保存动作由用户确认。
- 大文件 Diff 使用有界算法和渲染上限。
- JSON Schema 外部引用不会通过网络加载。

## 隐私

Orthos 不需要账号，不依赖云服务，也不上传文件内容。文件读取、校验、修正与保存均在本机完成。

## 从 LintDrop 升级

Orthos 是 LintDrop 的新名称。v1.1.0 保留原应用标识符 `com.lintdrop.desktop`，用于延续既有安装与升级识别；这是兼容性标识，不代表旧品牌仍在使用。功能、文件处理边界和使用方式保持连续。

## 从源码运行

需要 Rust stable、Node.js 18+、Visual Studio Build Tools 2022，以及 Windows 10 SDK 10.0.19041.0 或更高版本。

```powershell
git clone https://github.com/rowanjove/Orthos.git
cd Orthos
npm ci
npm run check
npx tauri dev
```

完整质量检查包含前端测试与语法检查、Rustfmt、Clippy、Rust 测试和 `cargo check`。构建 Windows 安装版：

```powershell
npx tauri build --bundles nsis
```

## 项目结构

- `src/`：HTML、CSS 和 JavaScript 界面。
- `test/`：前端回归测试。
- `src-tauri/src/`：Rust 校验流程与格式解析器。
- `src-tauri/benches/`：解析器基准。
- `src-tauri/windows/`：Windows 安装器钩子。

提交改动前请阅读 [贡献指南](CONTRIBUTING.md)，补充必要测试并运行 `npm run check`。

## 开源许可

Orthos 使用 [MIT License](LICENSE) 开源。
