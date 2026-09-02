# Orthos — Offline configuration validation and safe repair

[简体中文](README.md) | [English](README.en.md)

Orthos is a local Windows desktop validator for JSON, YAML, TOML, XML, CSV, INI, and ENV files. Drop a file or paste text to inspect error locations, Chinese-language diagnostics, and a before-and-after diff. Processing stays on your computer, repaired output must pass a second parser check, and original files are never overwritten automatically.

[Download for Windows](https://github.com/rowanjove/Orthos/releases/latest) · [Changelog](CHANGELOG.md) · [Report an issue](https://github.com/rowanjove/Orthos/issues)

[![CI](https://github.com/rowanjove/Orthos/actions/workflows/ci.yml/badge.svg)](https://github.com/rowanjove/Orthos/actions/workflows/ci.yml)
[![GitHub Release](https://img.shields.io/github/v/release/rowanjove/Orthos)](https://github.com/rowanjove/Orthos/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Download and use

The current version is **v1.1.0** for Windows 10/11 x64:

- `Orthos_1.1.0_x64-setup.exe`: installer.
- `Orthos_1.1.0_x64_portable.exe`: standalone executable.
- `SHA256SUMS.txt`: SHA-256 checksums for both downloads.

Download a build from [Releases](https://github.com/rowanjove/Orthos/releases/latest), then drop a configuration file or paste text. Review the diagnostics and, when a repair is available, inspect the diff before saving a new file.

## What it checks

| Capability | Details |
| --- | --- |
| Seven formats | JSON, YAML, TOML, XML, CSV, INI, ENV |
| Detection | Automatic from extension or content, with manual selection |
| Diagnostics | Line and column, nearby source, and Chinese explanations |
| Safe repair | Common syntax repairs, diff preview, and second validation pass |
| Batch workflow | Validate multiple files and save repaired outputs in batches |
| JSON Schema | Additional JSON structure checks without fetching external references |

Limits are **10 MB per file** and **20 MB per batch**.

## Safety boundaries

Orthos checks syntax and structure, not application-specific meaning. A valid port number does not mean it is appropriate for your deployment.

- Duplicate keys remain for manual resolution.
- Extra CSV fields are not silently deleted.
- Repaired output must pass the matching parser before it can be saved.
- Original files are not overwritten automatically.
- Large diffs use bounded computation and rendering.
- External JSON Schema references are not fetched over the network.

## Privacy

Orthos requires no account or cloud service. Reading, validation, repair, and saving happen locally; file contents are not uploaded.

## Upgrading from LintDrop

Orthos is the new name of LintDrop. Version 1.1.0 retains the application identifier `com.lintdrop.desktop` so existing installations keep a continuous upgrade identity. This is a compatibility identifier, not active branding.

## Run from source

Requires Rust stable, Node.js 18+, Visual Studio Build Tools 2022, and Windows 10 SDK 10.0.19041.0 or later.

```powershell
git clone https://github.com/rowanjove/Orthos.git
cd Orthos
npm ci
npm run check
npx tauri dev
```

The full check includes frontend tests and syntax checks, Rustfmt, Clippy, Rust tests, and `cargo check`. Build the Windows installer with `npx tauri build --bundles nsis`.

## License

Orthos is open source under the [MIT License](LICENSE).
