# LintDrop

轻量、离线、面向日常配置文件的桌面格式检查器。  
Lightweight, offline desktop validation for everyday configuration files.

LintDrop 是一款基于 Tauri 2.x 构建的轻量级桌面格式检查器，用来快速校验和修复常见配置文件。当 JSON、YAML、TOML、XML、CSV、INI、ENV 文件因为语法问题无法被程序加载时，它可以直接给出定位、解释和可预览的修正结果。

LintDrop is a Tauri 2.x desktop app for quickly checking and repairing common configuration-file formats. It is designed for the small but frequent moments when a JSON, YAML, TOML, XML, CSV, INI, or ENV file refuses to load and you want a clear answer without opening a full IDE.

## 本次更新

- 自动修复不再静默删除 YAML、TOML、INI 的重复键，也不会截断 CSV 的多余字段。
- 只有通过二次校验的修正结果才会出现在保存入口，避免保存仍然无效的内容。
- 大文件 Diff 使用有界算法和渲染上限，降低卡顿与内存占用风险。
- JSON Schema 会一次报告全部校验问题，并保持离线运行。
- 新增前端回归测试、Rust 解析器基准和 GitHub Actions 质量门禁。

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
  - YAML Tab 缩进、冒号后缺空格、重复 key 检测、空列表项、flow list/map 空格问题；重复 key 会保留原文并提示处理。
  - TOML 未闭合 section、缺失等号、重复 key 检测、未闭合字符串、含空格 key；重复 key 不会被静默删除。
  - XML 未闭合标签、标签不匹配、未闭合注释/CDATA、未加引号属性。
  - CSV 引号未闭合、列数不一致、常见分隔符和 BOM/CRLF。
  - INI/ENV 缺失等号、无效 key、重复项、未闭合引号；INI 重复项会保留原文并提示处理。
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
  - YAML tab indentation, missing spaces after colons, duplicate-key detection, empty list items, and flow list/map spacing. Duplicate keys are preserved and reported.
  - TOML unclosed sections, missing equals signs, duplicate-key detection, unclosed strings, and keys with spaces. Duplicate keys are never silently removed.
  - XML unclosed tags, mismatched tags, unclosed comments/CDATA, and unquoted attributes.
  - CSV unclosed quotes, inconsistent column counts, common delimiters, BOM, and CRLF.
  - INI/ENV missing equals signs, invalid keys, duplicate entries, and unclosed quotes. INI duplicates are preserved and reported.
- Shows a before/after diff before saving repaired files.
- Supports batch validation and batch saving.
- Includes optional JSON Schema validation.

### Privacy and Safety

LintDrop runs offline by default. It does not require an account, does not depend on a cloud service, and does not upload file contents.

Dragged files are read through a local Rust command so users can validate files from any project folder, drive, or external disk. File size limits and batch size limits remain in place to avoid accidentally loading oversized files.

### Boundaries

LintDrop focuses on syntax validation and common format repairs. It does not know whether your business-specific values are semantically correct, such as whether a port, endpoint, or internal field name is valid. For JSON, use JSON Schema when you need stricter structural validation.

## 安装 / Installation

从 GitHub Releases 下载适合你机器的 Windows 版本：

Download the Windows version that fits your machine from GitHub Releases:

[LintDrop Releases](https://github.com/rowanjove/lintdrop/releases)

当前 Windows 下载项 / Current Windows downloads：

```text
LintDrop_1.0.3_x64-setup.exe
LintDrop_1.0.3_x64_portable.exe
```

- `LintDrop_1.0.3_x64-setup.exe` —— 安装版，适合较老的系统，或希望按安装向导完成配置的用户。

  Installer build, recommended for older systems or users who want guided setup.

- `LintDrop_1.0.3_x64_portable.exe` —— 免安装版，适合 Windows 10/11；下载后双击即可运行。

  Portable build for Windows 10/11; download and double-click to run.

## 开发 / Development

环境要求 / Requirements：

- Rust stable
- Node.js 18+
- Visual Studio Build Tools 2022
- Windows 10 SDK 10.0.19041.0 or newer

安装依赖 / Install dependencies：

```bash
npm install
```

执行完整本地质量检查 / Run the full local quality gate：

```bash
npm run check
```

该命令会执行前端测试与语法检查、Rust 格式检查、Clippy、全部 Rust 测试和 `cargo check`。

This runs frontend tests and syntax checks, Rust formatting, Clippy, all Rust tests, and `cargo check`.

运行解析器基准 / Run the parser smoke benchmark：

```bash
npm run benchmark:rust
```

该基准使用固定语料，适合本地比较修改前后的趋势，不作为发布性能承诺。

The benchmark is deterministic and intended for comparing changes locally, not as a release performance promise.

开发模式运行 / Run in development mode：

```bash
npx tauri dev
```

构建 Windows 安装包 / Build the Windows installer：

```bash
npx tauri build --bundles nsis
```

免安装可执行文件 / Portable executable：

```text
src-tauri/target/release/lintdrop.exe
```

构建产物 / Build artifacts：

```text
src-tauri/target/release/bundle/
```

## 项目结构 / Project Structure

```text
lintdrop/
├── src/                    # 前端：HTML、CSS、JavaScript 与 Diff 工具
├── test/                   # 前端回归测试
├── src-tauri/              # Tauri/Rust 后端
│   ├── src/lib.rs          # 命令、校验流程与测试
│   ├── src/parsers/        # 各格式解析器与修复器
│   ├── benches/            # 解析器基准
│   ├── icons/              # 应用、安装器与卸载器图标
│   └── windows/            # NSIS 安装器钩子
├── .github/workflows/      # CI 质量门禁
├── ABOUT.md
├── LICENSE
├── README.md
└── package.json
```

## 验证 / Verification

发布构建使用以下命令检查：

```bash
npm run check
npx tauri build --bundles nsis
```

1.0.3 版本已在本地安装验证：主程序图标、红色卸载器图标、开始菜单和桌面快捷方式图标均正常。

The 1.0.3 release was also installed locally to verify the main executable icon, the red uninstall icon, and Start Menu/Desktop shortcut icon bindings.

## 许可证 / License

MIT
