# LintDrop

轻量、离线、中文友好的桌面配置文件格式检查器。

[![CI](https://github.com/rowanjove/lintdrop/actions/workflows/ci.yml/badge.svg)](https://github.com/rowanjove/lintdrop/actions/workflows/ci.yml)
[![GitHub Release](https://img.shields.io/github/v/release/rowanjove/lintdrop)](https://github.com/rowanjove/lintdrop/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

LintDrop 用来快速检查和修复 JSON、YAML、TOML、XML、CSV、INI、ENV 文件。把文件拖进窗口，或直接粘贴一段配置，即可查看中文错误说明、具体位置和安全的修正预览。应用默认离线运行，不上传文件，也不会直接覆盖原文件。

> English: LintDrop is a lightweight, offline desktop validator for common configuration files. See the [short English introduction](#english) below.

## 下载

请从 [GitHub Releases](https://github.com/rowanjove/lintdrop/releases/latest) 下载最新 Windows 版本：

- `LintDrop_1.0.4_x64-setup.exe`：安装版，适合希望使用安装向导的用户。
- `LintDrop_1.0.4_x64_portable.exe`：免安装版，适合 Windows 10/11，下载后双击即可运行。

## v1.0.4 更新内容

- 自动修复不再静默删除 YAML、TOML、INI 的重复键。
- CSV 修复不再截断多余字段，避免意外丢失数据。
- 修正结果会再次校验，只有有效内容才会提供保存入口。
- 大文件 Diff 使用有界算法和渲染上限，降低卡顿与内存占用风险。
- JSON Schema 一次报告全部校验问题，并禁用外部网络引用。
- 主页和提示文案改为中文优先，保存、批量检查与错误反馈更加直观。
- 新增前端回归测试、Rust 解析器基准和 GitHub Actions 质量门禁。

## 功能

- 支持拖拽文件、点击选择文件和粘贴文本。
- 支持 JSON、YAML、TOML、XML、CSV、INI、ENV 七种格式。
- 自动识别格式，粘贴内容时也可以手动指定。
- 提供行列级错误定位、附近原文和中文说明。
- 自动修正常见语法问题，并在保存前显示 Diff。
- 支持多文件批量检查和批量保存修正结果。
- 支持使用 JSON Schema 进一步检查 JSON 结构。
- 单文件最大 10 MB，批量文件合计最大 20 MB。

## 安全原则

LintDrop 的自动修复以“不丢数据”为优先原则：

- 重复键会保留原文并提示手动处理，不猜测应该保留哪一个值。
- CSV 多余字段不会被自动删除。
- 修复后的内容必须再次通过对应格式的解析器。
- 原文件不会被自动覆盖，只有用户确认后才会保存新文件。
- JSON Schema 外部引用不会联网加载。

## 隐私

LintDrop 默认完全离线运行，不需要账号，不依赖云服务，也不会上传文件内容。文件读取、检查、修正和保存都在本机完成。

## 能力边界

LintDrop 关注语法和结构是否有效，不理解业务语义。例如，它可以检查端口字段是不是合法 JSON 数字，但不会判断这个端口是否符合你的业务规则。JSON 文件可以配合 JSON Schema 添加更严格的结构约束。

## 本地开发

环境要求：

- Rust stable
- Node.js 18 或更高版本
- Visual Studio Build Tools 2022
- Windows 10 SDK 10.0.19041.0 或更高版本

安装依赖：

```bash
npm install
```

运行完整质量检查：

```bash
npm run check
```

该命令包含前端测试与语法检查、Rust 格式检查、Clippy、全部 Rust 测试和 `cargo check`。

启动开发模式：

```bash
npx tauri dev
```

构建 Windows 安装版：

```bash
npx tauri build --bundles nsis
```

运行解析器基准：

```bash
npm run benchmark:rust
```

## 项目结构

```text
lintdrop/
├── src/                    # HTML、CSS、JavaScript 前端
├── test/                   # 前端回归测试
├── src-tauri/              # Tauri/Rust 后端
│   ├── src/lib.rs          # 命令、校验流程与测试
│   ├── src/parsers/        # 各格式解析器与修复器
│   ├── benches/            # 解析器基准
│   ├── icons/              # 应用与安装器图标
│   └── windows/            # NSIS 安装器钩子
├── .github/workflows/      # GitHub Actions
├── CONTRIBUTING.md         # 贡献指南
├── LICENSE                 # MIT 许可证
└── README.md
```

## 参与贡献

欢迎提交 Issue 和 Pull Request。开始前请阅读 [贡献指南](CONTRIBUTING.md)。修复 Bug 时建议同时补充回归测试，提交前请确保 `npm run check` 全部通过。

- [报告问题](https://github.com/rowanjove/lintdrop/issues/new)
- [查看 Issues](https://github.com/rowanjove/lintdrop/issues)
- [提交 Pull Request](https://github.com/rowanjove/lintdrop/pulls)

## 开源许可证

LintDrop 使用 [MIT License](LICENSE) 开源。你可以自由使用、复制、修改、合并、发布和分发本项目，但需要保留许可证和版权声明。

## English

LintDrop is a lightweight, offline desktop validator and safe repair tool for JSON, YAML, TOML, XML, CSV, INI, and ENV files. It provides line-level diagnostics, diff previews, batch validation, and optional JSON Schema checks. The original file is never overwritten automatically. Download the latest Windows installer or portable build from [GitHub Releases](https://github.com/rowanjove/lintdrop/releases/latest).
