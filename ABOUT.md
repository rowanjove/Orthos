# About LintDrop / 关于 LintDrop

## 中文

LintDrop 是一款轻量、离线的桌面格式检查器，专注于日常配置文件的快速校验和常见语法问题修复。

它支持 JSON、YAML、TOML、XML、CSV、INI、ENV 七种格式。用户可以把文件拖进窗口，也可以直接粘贴文本。LintDrop 会自动识别格式，定位错误行列，给出中文友好的错误说明，并在能够安全推断时生成修正版本。

LintDrop 的修正不会直接覆盖原文件。你会先看到修正前后的 Diff，再决定是否保存。多文件场景下，它也支持批量校验和批量保存修正结果。

LintDrop 默认离线运行，不需要登录，不上传文件内容。它适合处理本地项目、公司 workspace、下载目录、外接盘和临时配置片段。

需要注意的是，LintDrop 主要关注语法和结构。它可以告诉你文件是否能被解析，也可以修正常见格式错误，但不会判断业务字段是否符合你的内部规则。对于 JSON，可以使用 JSON Schema 做进一步约束。

## English

LintDrop is a lightweight, offline desktop format checker focused on quick validation and repair of everyday configuration files.

It supports JSON, YAML, TOML, XML, CSV, INI, and ENV. Users can drop files into the window or paste text directly. LintDrop detects the format, reports line and column diagnostics, explains errors in friendly Chinese, and generates a repaired version when the issue can be fixed safely.

Repairs never overwrite the original file automatically. LintDrop shows a before/after diff first, then lets you decide whether to save the repaired content. Batch validation and batch saving are supported for multi-file workflows.

LintDrop runs offline by default. It does not require an account and does not upload file contents. It is useful for local projects, company workspaces, download folders, external drives, and temporary configuration snippets.

LintDrop focuses on syntax and structure. It can tell whether a file can be parsed and repair common format mistakes, but it does not validate business-specific semantics. For JSON, use JSON Schema when stricter structural rules are needed.

## Version / 版本

1.0.3

## Formats / 支持格式

JSON / YAML / TOML / XML / CSV / INI / ENV
