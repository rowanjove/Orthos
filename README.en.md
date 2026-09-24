# Orthos

> **Local structured configuration validator and workbench. 100% offline, zero data upload, no network requests.**

[简体中文](README.md) | [English](README.en.md) · [Latest Release](https://github.com/rowanjove/Orthos/releases/latest) · [Changelog](CHANGELOG.md) · [Issues](https://github.com/rowanjove/Orthos/issues)

[![CI](https://github.com/rowanjove/Orthos/actions/workflows/ci.yml/badge.svg)](https://github.com/rowanjove/Orthos/actions/workflows/ci.yml)
[![GitHub Release](https://img.shields.io/github/v/release/rowanjove/Orthos)](https://github.com/rowanjove/Orthos/releases/latest)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](LICENSE)

![Orthos Workbench UI](docs/images/orthos-main.png)

When troubleshooting services or production incidents, broken configurations caused by missing commas, misplaced indents, or unmatched brackets can easily break deployments. Pasting configs containing database credentials or API secrets into online web formatters poses serious privacy and security risks.

Orthos is a desktop application that runs entirely offline. Drag in configuration files or paste text to pinpoint syntax errors in milliseconds, inspect AST structures, edit visually, and review text or semantic diffs before saving. Everything is processed locally without network access or telemetry.

---

## Features

- **15+ Formats Supported**:
  - JSON Ecosystem: JSON, JSONC, JSON5, JSONL / NDJSON
  - Standard Configs: YAML, TOML, XML, INI, ENV
  - Data & Properties: CSV, TSV, Java Properties
  - Developer Toolchains: EditorConfig, GitConfig, HCL / Terraform
- **Multi-Tab Workbench**: Open, inspect, edit, and save multiple files independently.
- **Multiple Editor Modes**:
  - Source Code Editor: Real-time linting, line gutter, cursor positioning, and `Ctrl+S` safe saving.
  - Visual Tree View: Universal AST tree representation with add, delete, and modify capabilities.
  - Schema Form: Dynamically generated interactive form inputs based on JSON Schemas.
  - Dual-Mode Diff: Line-level text diff (LCS) and AST semantic diff.
- **Format Sniffing & Formatting**: Heuristic format detection and one-click standard formatting.
- **Safe Auto-Repair**: Automatically fix common mistakes (trailing commas, quotes, brackets). Repaired content must pass strict re-parsing before saving.
- **Profile Diagnostics & Templates**: Built-in specifications (e.g., `package.json`, `docker-compose.yml`) and ready-to-use templates.

---

## Security Principles

- **Local Only**: No telemetry, no external HTTP calls, no cloud backend.
- **Secondary Verification Gate**: Auto-fixed content is verified against native parsers before allowing saves.
- **Offline Schema**: JSON Schema evaluation executes strictly in-memory without remote network references.
- **Directory Traversal Protection**: File saving is restricted to safe directory paths.

---

## Downloads (Windows 10 / 11 x64)

- **`Orthos_2.0.0_x64-setup.exe`**: Standard installer with start menu and desktop shortcuts.
- **`Orthos_2.0.0_x64_zh-CN.msi`**: MSI package for enterprise or automated deployment.
- **`orthos.exe`**: Standalone portable executable.

Download them from the **[GitHub Releases Page](https://github.com/rowanjove/Orthos/releases/latest)**.

---

## Development & Build

### Requirements
- Node.js 18+
- Rust stable (1.80+)
- Visual Studio C++ Build Tools (Windows)

### Commands
```powershell
# Install dependencies
npm install

# Run all checks & tests (frontend + Rust)
npm run check

# Start desktop dev mode
npm run tauri dev

# Build release packages
npm run build
```

---

## License

Orthos is open source under the [Apache-2.0 License](LICENSE).
