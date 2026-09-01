# LintDrop — Offline configuration validation and repair

[简体中文](README.md) | [English](README.en.md)

LintDrop is an offline Windows desktop validator for JSON, YAML, TOML, XML, CSV, INI, and ENV files. Drop a file or paste text to inspect error locations, Chinese-language diagnostics, and a before-and-after diff. Saving repaired output requires confirmation; the original file is not overwritten automatically.

[Download for Windows](https://github.com/rowanjove/lintdrop/releases/latest) · [Changelog](CHANGELOG.md) · [Report an issue](https://github.com/rowanjove/lintdrop/issues)

[![CI](https://github.com/rowanjove/lintdrop/actions/workflows/ci.yml/badge.svg)](https://github.com/rowanjove/lintdrop/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Install and use

The current version is **v1.0.4**, with Windows 10/11 x64 downloads:

- `LintDrop_1.0.4_x64-setup.exe`: installer.
- `LintDrop_1.0.4_x64_portable.exe`: standalone executable.

Download from [Releases](https://github.com/rowanjove/lintdrop/releases/latest), start the application, and drop a configuration file or paste text. Check the detected format and diagnostics. If a repair is available, review the diff before saving a new file.

## Validation features

| Capability | Details |
| --- | --- |
| Seven formats | JSON, YAML, TOML, XML, CSV, INI, ENV |
| Format detection | Automatic detection; manual selection for pasted content |
| Error locations | Line and column, nearby source, and Chinese-language explanations |
| Repair preview | Common syntax repairs with a diff and a second validation pass |
| Batch processing | Validate multiple files and save repaired outputs in batches |
| JSON Schema | Additional JSON structure checks without fetching external references |

Limits are **10 MB per file** and **20 MB per batch**.

## Repair boundaries

LintDrop checks syntax and structure, not application-specific semantics. A valid JSON number, for example, does not establish that a port is appropriate for your deployment.

- Duplicate keys remain for manual resolution; the tool does not guess which value to keep.
- Extra CSV fields are not deleted to make validation pass.
- Repaired output must pass its format parser again before saving becomes available.
- Original files are not automatically overwritten; confirmed repairs are saved as new files.
- Large-file diffs use bounded algorithms and rendering limits, not unlimited interactive comparison.

See [CHANGELOG.md](CHANGELOG.md) for the v1.0.4 repair and quality-check changes.

## Privacy

File reading, validation, repair, and saving happen locally. No account is required and file contents are not uploaded. External network references in JSON Schema are disabled. The application is designed primarily for Chinese users; this English README does not imply that every diagnostic is localized into English.

## Run from source

Requires Rust stable, Node.js 18+, Visual Studio Build Tools 2022, and Windows 10 SDK 10.0.19041.0 or later.

```powershell
git clone https://github.com/rowanjove/lintdrop.git
cd lintdrop
npm ci
npm run check
npx tauri dev
```

`npm run check` covers frontend tests and syntax checks, Rust formatting, Clippy, Rust tests, and `cargo check`.

Build the installer and run parser benchmarks:

```powershell
npx tauri build --bundles nsis
npm run benchmark:rust
```

## Code and contributions

- `src/`: HTML, CSS, and JavaScript interface.
- `test/`: frontend regression tests.
- `src-tauri/src/`: Rust validation flow and format parsers.
- `src-tauri/benches/`: parser benchmarks.
- `src-tauri/windows/`: installer hooks.

Read the [contribution guide (Chinese)](CONTRIBUTING.md), include regression tests for fixes, and run `npm run check` before submitting changes.

## License

Available under the [MIT License](LICENSE).
