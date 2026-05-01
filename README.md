# LintDrop

轻量、离线、面向日常配置文件的桌面格式检查器。  
Lightweight, offline desktop validation for everyday configuration files.

LintDrop is a Tauri 2.x desktop app for quickly checking and repairing common configuration-file formats. It is designed for the small but frequent moments when a JSON, YAML, TOML, XML, CSV, INI, or ENV file refuses to load and you want a clear answer without opening a full IDE.

LintDrop 是一款基于 Tauri 2.x 构建的轻量级桌面格式检查器，用来快速校验和修复常见配置文件。当 JSON、YAML、TOML、XML、CSV、INI、ENV 文件因为语法问题无法被程序加载时，它可以直接给出定位、解释和可预览的修正结果。

## 中文

### 适合谁

- 开发者：快速检查项目配置、CI 配置、环境变量文件和数据导入文件。
- 运维/测试/产品同学：在不写代码的情况下检查配置文件是否可用。
- 日常办公场景：处理下载、导出、复制来的配置片段或表格文本。

### 核心能力

- 支持拖拽文件、点击选择文件、粘贴文本三种输入方式。
- 支持 JSON、YAML、TOML、XML、CSV、INI、ENV 七种格式。
- 自动识别格式，并在必要时允许手动选择。
- 错误定位精确到行列号，并展示出错附近内容。
- 中文友好的错误解释，尽量把解析器错误翻译成人能读懂的提示。
- 自动修正常见问题，例如：
  - JSON 注释、单引号、未加引号的 key、`undefined`、末尾逗号、缺失括号。
  - YAML Tab 缩进、冒号后缺空格、重复 key、空列表项、flow list/map 空格问题。
  - TOML 未闭合 section、缺失等号、重复 key、未闭合字符串、含空格 key。
  - XML 未闭合标签、标签不匹配、未闭合注释/CDATA、未加引号属性。
  - CSV 引号未闭合、列数不一致、常见分隔符和 BOM/CRLF。
  - INI/ENV 缺失等号、无效 key、重复项、未闭合引号。
- 修正前后 Diff 预览，不会直接覆盖原文件。
- 多文件批量校验和批量保存修正结果。
- JSON Schema 可选校验。

### 隐私和安全

LintDrop 默认离线运行，不需要登录，不依赖云服务，也不会上传文件内容。

拖拽读取由本地 Rust 命令处理，目的是允许用户检查任意工作区、磁盘分区或外接盘中的配置文件。读取流程保留文件大小和批量总大小限制，用来避免误拖入超大文件。

### 能力边界

LintDrop 主要解决“格式是否能解析”和“常见语法错误能否快速修复”。它不会理解业务语义，例如端口号是否正确、字段是否符合你公司的内部规范，除非你使用 JSON Schema 对 JSON 内容做额外约束。

## English

### Who It Is For

- Developers who need to inspect project config, CI config, environment files, and import data quickly.
- Operations, QA, product, or support teammates who need readable validation without writing code.
- Everyday desktop workflows involving downloaded, exported, or copied configuration snippets.

### Core Features

- Drag files, select files, or paste text.
- Supports JSON, YAML, TOML, XML, CSV, INI, and ENV.
- Detects file formats automatically, with manual selection for pasted content.
- Reports line and column level diagnostics with nearby source context.
- Provides friendly Chinese explanations for parser errors.
- Repairs common issues, including:
  - JSON comments, single quotes, unquoted keys, `undefined`, trailing commas, and missing brackets.
  - YAML tab indentation, missing spaces after colons, duplicate keys, empty list items, and flow list/map spacing.
  - TOML unclosed sections, missing equals signs, duplicate keys, unclosed strings, and keys with spaces.
  - XML unclosed tags, mismatched tags, unclosed comments/CDATA, and unquoted attributes.
  - CSV unclosed quotes, inconsistent column counts, common delimiters, BOM, and CRLF.
  - INI/ENV missing equals signs, invalid keys, duplicate entries, and unclosed quotes.
- Shows a before/after diff before saving repaired files.
- Supports batch validation and batch saving.
- Includes optional JSON Schema validation.

### Privacy and Safety

LintDrop runs offline by default. It does not require an account, does not depend on a cloud service, and does not upload file contents.

Dragged files are read through a local Rust command so users can validate files from any project folder, drive, or external disk. File size limits and batch size limits remain in place to avoid accidentally loading oversized files.

### Boundaries

LintDrop focuses on syntax validation and common format repairs. It does not know whether your business-specific values are semantically correct, such as whether a port, endpoint, or internal field name is valid. For JSON, use JSON Schema when you need stricter structural validation.

## Installation / 安装

Download the Windows installer from GitHub Releases:  
从 GitHub Releases 下载 Windows 安装包：

[LintDrop Releases](https://github.com/rowanjove/lintdrop/releases)

Current installer / 当前安装包：

```text
LintDrop_1.0.3_x64-setup.exe
```

## Development / 开发

Requirements / 环境要求：

- Rust stable
- Node.js 18+
- Visual Studio Build Tools 2022
- Windows 10 SDK 10.0.19041.0 or newer

Install dependencies / 安装依赖：

```bash
npm install
```

Run in development mode / 开发模式运行：

```bash
npx tauri dev
```

Build the Windows installer / 构建 Windows 安装包：

```bash
npx tauri build --bundles nsis
```

Build artifacts / 构建产物：

```text
src-tauri/target/release/bundle/
```

## Project Structure / 项目结构

```text
lintdrop/
├── src/                    # Frontend: HTML, CSS, JavaScript
├── src-tauri/              # Tauri/Rust backend
│   ├── src/lib.rs          # Commands, validation flow, tests
│   ├── src/parsers/        # Format-specific parsers and fixers
│   ├── icons/              # App, installer, and uninstall icons
│   └── windows/            # NSIS installer hooks
├── ABOUT.md
├── LICENSE
├── README.md
└── package.json
```

## Verification / 验证

Release builds are checked with:

```bash
node --check src/main.js
cargo test
cargo clippy -- -D warnings
npx tauri build --bundles nsis
```

The 1.0.3 release was also installed locally to verify the main executable icon, the red uninstall icon, and Start Menu/Desktop shortcut icon bindings.

1.0.3 版本已在本地安装验证：主程序图标、红色卸载器图标、开始菜单和桌面快捷方式图标均正常。

## License / 许可证

MIT
