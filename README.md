# LintDrop — 离线配置文件检查与修复

[简体中文](README.md) | [English](README.en.md)

LintDrop 是 Windows 上的离线配置文件检查器，支持 JSON、YAML、TOML、XML、CSV、INI 和 ENV。拖入文件或粘贴文本，即可查看错误位置、中文说明与修正前后 Diff；保存修正结果需要用户确认，不会自动覆盖原文件。

[下载 Windows 版](https://github.com/rowanjove/lintdrop/releases/latest) · [更新记录](CHANGELOG.md) · [报告问题](https://github.com/rowanjove/lintdrop/issues)

[![CI](https://github.com/rowanjove/lintdrop/actions/workflows/ci.yml/badge.svg)](https://github.com/rowanjove/lintdrop/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## 安装与使用

当前版本为 **v1.0.4**，提供 Windows 10/11 x64 版本：

- `LintDrop_1.0.4_x64-setup.exe`：安装向导。
- `LintDrop_1.0.4_x64_portable.exe`：免安装程序。

从 [Releases](https://github.com/rowanjove/lintdrop/releases/latest) 下载后，启动程序，拖入配置文件或粘贴文本。检查格式识别结果，阅读诊断；有可用修正时先查看 Diff，再决定是否保存新文件。

## 可以检查什么

| 能力 | 说明 |
| --- | --- |
| 七种格式 | JSON、YAML、TOML、XML、CSV、INI、ENV |
| 格式识别 | 自动识别；粘贴内容时也可手动选择 |
| 错误定位 | 行列位置、附近原文和中文说明 |
| 修复预览 | 修正常见语法问题，展示 Diff，并再次校验 |
| 批量处理 | 多文件检查和批量保存修正结果 |
| JSON Schema | 为 JSON 增加结构约束；不联网加载外部引用 |

单文件上限为 **10 MB**，批量合计上限为 **20 MB**。

## 修复边界

LintDrop 关注语法和结构，不判断配置是否符合具体业务。例如，合法的 JSON 数字不代表端口设置符合你的部署要求。

- 不自动选择重复键应保留的值，保留原文并提示手动处理。
- 不通过删除 CSV 多余字段来使文件“通过检查”。
- 修正结果必须再次通过对应格式的解析器，才提供保存入口。
- 原文件不被自动覆盖，用户确认后保存新文件。
- 大文件 Diff 有算法和渲染上限，不承诺无限大小的交互式比较。

v1.0.4 的修复与质量检查变化见 [CHANGELOG.md](CHANGELOG.md)。

## 隐私

文件读取、检查、修正与保存都在本机完成，不需要账号，也不上传文件内容。JSON Schema 的外部网络引用被禁用。应用以中文使用体验为主，英文 README 不代表所有错误信息已英文化。

## 从源码运行

需要 Rust stable、Node.js 18+、Visual Studio Build Tools 2022，以及 Windows 10 SDK 10.0.19041.0 或更高版本。

```powershell
git clone https://github.com/rowanjove/lintdrop.git
cd lintdrop
npm ci
npm run check
npx tauri dev
```

`npm run check` 包括前端测试和语法检查、Rust 格式检查、Clippy、Rust 测试及 `cargo check`。

构建安装包与运行解析器基准：

```powershell
npx tauri build --bundles nsis
npm run benchmark:rust
```

## 代码与贡献

- `src/`：HTML、CSS 和 JavaScript 界面。
- `test/`：前端回归测试。
- `src-tauri/src/`：Rust 校验流程与格式解析器。
- `src-tauri/benches/`：解析器基准。
- `src-tauri/windows/`：安装器钩子。

提交前阅读 [贡献指南](CONTRIBUTING.md)，为修复补充回归测试并运行 `npm run check`。

## 许可

采用 [MIT License](LICENSE)。
