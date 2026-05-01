# LintDrop

极轻量桌面配置文件格式检查器。  
A lightweight desktop linter for everyday configuration files.

LintDrop 基于 Tauri 2.x 构建，支持拖拽文件或粘贴内容进行即时校验，提供行级错误定位、中文友好提示、Diff 预览和一键保存修正文件。应用离线运行，无需联网。

LintDrop is built with Tauri 2.x. It validates dropped files or pasted text instantly, shows line-level diagnostics, friendly Chinese messages, diff previews, and one-click saving of repaired files. It runs fully offline.

## 支持格式 / Supported Formats

JSON / YAML / TOML / XML / CSV / INI / ENV

## 功能 / Features

- 拖拽任意本地路径文件，或点击选择文件  
  Drag files from any local path, or select files manually.
- 粘贴内容并自动识别格式  
  Paste content and let LintDrop detect the format.
- 精确到行列号的错误定位  
  Line and column level diagnostics.
- 中文友好错误解释  
  Friendly Chinese explanations for common syntax issues.
- 自动修正常见问题：未闭合引号、缩进错误、重复键、末尾逗号、未闭合标签等  
  Auto-fixes common issues such as unclosed quotes, indentation mistakes, duplicate keys, trailing commas, and unclosed tags.
- 修正前后 Diff 对比  
  Before/after diff preview.
- 多文件批量校验和批量保存修正文件  
  Batch validation and batch saving for repaired files.
- 可选 JSON Schema 校验  
  Optional JSON Schema validation.
- 离线运行，不上传文件内容  
  Offline-first. File contents are not uploaded.

## 安装 / Installation

Windows 安装包在发布页下载：  
Download the Windows installer from Releases:

```text
LintDrop_1.0.3_x64-setup.exe
```

## 开发环境 / Development Requirements

- Rust stable
- Node.js 18+
- Visual Studio Build Tools 2022
- Windows 10 SDK 10.0.19041.0 or newer

## 本地开发 / Local Development

```bash
npm install
npx tauri dev
```

## 构建 / Build

```bash
npx tauri build --bundles nsis
```

构建产物位于：  
Build artifacts are generated under:

```text
src-tauri/target/release/bundle/
```

## 项目结构 / Project Structure

```text
lintdrop/
├── src/                    # Frontend: HTML + CSS + JS
├── src-tauri/              # Tauri/Rust backend
│   ├── src/lib.rs          # Commands and validation flow
│   ├── src/parsers/        # Format parsers and fixers
│   └── icons/              # App and installer icons
├── ABOUT.md
├── README.md
└── package.json
```

## 隐私 / Privacy

LintDrop 默认在本机处理文件内容，不需要网络连接，也不会上传文件。  
LintDrop processes files locally by default, does not require a network connection, and does not upload file contents.

## License

MIT
