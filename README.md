# Orthos

> **本地结构化配置检查与编辑工作台。100% 离线运行，配置不上传、无网络外联。**

[简体中文](README.md) | [English](README.en.md) · [下载最新版本](https://github.com/rowanjove/Orthos/releases/latest) · [更新日志](CHANGELOG.md) · [反馈问题](https://github.com/rowanjove/Orthos/issues)

[![CI](https://github.com/rowanjove/Orthos/actions/workflows/ci.yml/badge.svg)](https://github.com/rowanjove/Orthos/actions/workflows/ci.yml)
[![GitHub Release](https://img.shields.io/github/v/release/rowanjove/Orthos)](https://github.com/rowanjove/Orthos/releases/latest)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](LICENSE)

![Orthos 工作台界面](docs/images/orthos-main.png)

平时调试服务或排查线上故障时，常常遇到配置文件因为漏写逗号、缩进错位或引号未闭合而导致程序起不来。如果配置里包含数据库账号、生产密钥或私有云 API Key，直接复制粘贴到公网的在线格式化网页又存在泄露隐患。

Orthos 是一个纯本地运行的桌面工具：拖入配置文件或直接粘贴文本，即可毫秒级定位语法错误，提供实时行号定位、多视图可视化编辑、安全修复及变更对比。所有解析均在本地进程完成，无任何网络依赖与外部数据收集。

---

## 主要功能

### 1. 支持 15+ 种常用格式
涵盖现代开发与运维常用的大多数配置文件：
- **JSON 生态**：JSON、JSONC（带注释）、JSON5、JSONL / NDJSON
- **常用配置**：YAML、TOML、XML、INI、ENV（`.env`）
- **数据与属性**：CSV、TSV、Java Properties
- **工具链配置**：EditorConfig（`.editorconfig`）、GitConfig（`.gitconfig`）、HCL / Terraform（`.tf`）

### 2. 多标签页工作台
支持同时打开或拖入多个配置文件，各自独立维护编辑状态、校验结果与未保存标记，支持单个或批量保存与关闭。

### 3. 多种编辑模式
- **代码视图**：带行号槽与实时光标位置显示，支持实时纠错与语法高亮，按 `Ctrl+S` 安全保存。
- **可视化树形视图**：将配置解析为统一 AST 树形节点，支持直观查看层级、添加/删除/修改节点。
- **Schema 动态表单**：针对配置 Schema 自动生成可视化表单输入项，适合引导式填写规范配置。
- **双模差异对比（Diff）**：
  - **Text Diff**：基于 LCS 算法展示修改前后的行级变更；
  - **Semantic Diff**：深入 AST 语义树，对比具体键路径（Path）、变更类型与值的前后差异。

### 4. 智能嗅探与安全修复
- **格式嗅探**：根据文件名后缀与内容结构特征自动判定格式，也支持手动随时切换。
- **一键排版**：按对应格式的标准缩进与规范一键重新排版。
- **安全修复**：针对单引号、漏逗号、未闭合括号等常见低级错误提供自动修正建议。修正后的内容必须先在内存中重新通过真实解析器校验，并支持 Diff 预览，绝不强制覆盖原文件。

### 5. Profile 规范诊断与模板
内置 `package.json`、`docker-compose.yml` 等场景的 Profile 结构规范诊断规则，自动检查必填字段、版本命名与依赖声明，并预置常见配置文件示例模板。

---

## 安全设计

- **数据不出机**：纯本地运行，不含任何数据统计、遥测日志或云端接口。
- **二次校验门禁**：所有自动生成的修复内容必须再次通过解析器验证才能保存。
- **离线 Schema**：JSON Schema 校验全程在本地内存执行，不加载任何外部网络 URL。
- **保存防穿透**：工作区文件保存限制在指定合法路径内，防止目录遍历逃逸。

---

## 下载安装

支持 64 位 Windows 10 / 11 系统：

| 发布包 | 说明 |
| :--- | :--- |
| **`Orthos_2.0.0_x64-setup.exe`** | 标准安装版（自动创建桌面与开始菜单快捷方式，支持完整卸载） |
| **`Orthos_2.0.0_x64_zh-CN.msi`** | 企业/批量部署安装包（Windows Installer 格式） |
| **`orthos.exe`** | 单文件免安装便携版（双击即用，不写系统注册表） |

可前往 **[GitHub Releases 页面](https://github.com/rowanjove/Orthos/releases/latest)** 直接下载对应文件。

---

## 本地开发与构建

### 环境要求
- Node.js 18+
- Rust stable (1.80+)
- Visual Studio C++ 生成工具 (Windows)

### 步骤
```powershell
# 1. 克隆代码仓库
git clone https://github.com/rowanjove/Orthos.git
cd Orthos

# 2. 安装前端依赖
npm install

# 3. 运行代码检查与全量测试套件
npm run check

# 4. 启动桌面端开发调试模式
npm run tauri dev

# 5. 打包生产安装包与发布产物
npm run build
```

打包产物位于 `src-tauri/target/release/` 及 `src-tauri/target/release/bundle/` 目录下。

---

## 许可证

本项目基于 [Apache-2.0 许可证](LICENSE) 开源。
