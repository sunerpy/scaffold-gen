# scaffold-gen

> 用一个交互式 CLI 为 Go、Rust、Python、TypeScript 生成可直接构建的项目脚手架。

[![CI](https://github.com/sunerpy/scaffold-gen/actions/workflows/ci.yml/badge.svg)](https://github.com/sunerpy/scaffold-gen/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/sunerpy/scaffold-gen)](https://github.com/sunerpy/scaffold-gen/releases)
[![Crates.io](https://img.shields.io/crates/v/scaffold-gen.svg)](https://crates.io/crates/scaffold-gen)
[![License](https://img.shields.io/crates/l/scaffold-gen.svg)](../../LICENSE)

简体中文 · [English](../../README.md)

`scafgen` 是一个快速、可扩展的脚手架生成器。选择语言和框架，回答几个提示，
即可得到一个带有合理默认值、LICENSE 和已初始化 git 仓库、可直接构建的项目。

## 目录

- [特性](#特性)
- [支持的框架](#支持的框架)
- [安装](#安装)
- [快速开始](#快速开始)
- [使用方法](#使用方法)
- [配合 LLM 使用](#配合-llm-使用)
- [架构设计](#架构设计)
- [开发](#开发)
- [贡献](#贡献)
- [许可证](#许可证)

## 特性

- **数据驱动注册表** —— `FrameworkSpec` 表 + `GenKind` 枚举调度；新增框架只需一行注册表记录 +
  模板目录，无需改代码。
- **交互式 CLI** —— 基于 `inquire` 的提示，覆盖语言、框架、端口、许可证。
- **内嵌模板** —— 所有模板编译进二进制，运行时无外部文件依赖。
- **环境验证** —— 生成前检查所需工具链（Go ≥ 1.24、Rust ≥ 1.88、Python ≥ 3.12）。
- **结构化日志** —— 静默（`-q`）/ 详细（`-v`）全局开关；输出经 `tracing` 写入 stderr。

## 支持的框架

| 语言       | 框架       | 状态                        |
| ---------- | ---------- | --------------------------- |
| Go         | Gin        | ✅                          |
| Go         | Go-Zero    | ⚠️ 未实现                   |
| Go         | MCP Server | ✅（streamable-HTTP + SSE） |
| Rust       | CLI App    | ✅                          |
| Rust       | Tauri      | ✅                          |
| TypeScript | Vue 3      | ✅（离线内嵌脚手架）        |
| TypeScript | React      | ✅（离线内嵌脚手架）        |
| Python     | Basic      | ✅                          |
| Python     | FastAPI    | ✅（配置驱动 API）          |
| Python     | MCP Server | ✅（FastMCP / 官方 SDK）    |

## 安装

**Linux / macOS**（shell）：

```sh
curl -fsSL https://raw.githubusercontent.com/sunerpy/scaffold-gen/main/scripts/install.sh | sh
```

**Windows**（PowerShell）：

```powershell
irm https://raw.githubusercontent.com/sunerpy/scaffold-gen/main/scripts/install.ps1 | iex
```

**通过 cargo**（任意安装了 Rust 的平台）：

```sh
cargo install scaffold-gen
```

**预编译二进制** —— Linux / macOS / Windows（x86_64 + aarch64）见
[Releases](https://github.com/sunerpy/scaffold-gen/releases) 页面。

**从源码构建**：`git clone … && cd scaffold-gen && make release`。

安装脚本支持 `TOOL_VERSION`（固定某个发布版本）和 `TOOL_INSTALL_DIR`
（自定义安装目录）环境变量覆盖。

## 快速开始

```sh
scafgen new my-project
```

CLI 会引导你完成语言、框架、项目配置和许可证选择，然后在 `./my-project`
中生成项目。

## 使用方法

```sh
# 交互模式（推荐）
scafgen new my-project

# 直接指定框架
scafgen new my-gin-app    --framework gin
scafgen new my-gozero-app --framework go-zero
scafgen new my-tauri-app  --framework tauri
scafgen new my-vue-app    --framework vue3
scafgen new my-react-app  --framework react
scafgen new my-api        --framework fastapi --language python
scafgen new my-mcp        --framework mcp-server --language go
scafgen new my-mcp-py     --framework mcp-python --language python
scafgen new my-mcp-py     --framework mcp-python --language python --mcp-backend official

# 同时生成 Makefile + Dockerfile（本地构建 / 容器化）
scafgen new my-gin-app --framework gin --with-build

# 安装 agent skill，然后重启 AI agent 即可自动通过 scafgen 引导新项目
scafgen skill install              # 安装到所有检测到的 agent（全局）
scafgen skill status               # 查看各 agent 安装状态
scafgen skill update --force       # 刷新为内嵌版本
scafgen skill uninstall            # 卸载
#   --target <opencode|claude|cursor|kiro>（可重复）  --global（默认）/ --local

# 全局开关（对所有子命令生效）
scafgen -q new my-project   # 静默：仅显示错误
scafgen -v new my-project   # 详细：显示调试输出
```

运行 `scafgen new --help` 查看完整选项列表。

## 配合 LLM 使用

先按 [安装](#安装) 章节完成安装，然后用以下命令驱动工具：

<details>
<summary>供 agent 调用的命令（非交互式）</summary>

- `scafgen new <name> --framework <fw> --language <lang>` —— 非交互式生成；通过 flag 传入所有选择以避免提示。
- `scafgen new --help` —— 查看可用 flag 和框架。
- `scafgen --version` —— 打印版本。

支持的框架：`gin`、`go-zero`、`mcp-server`、`tauri`、`vue3`、`react`、`fastapi`、`mcp-python`。

诊断信息和错误输出到 stderr；失败时以非零退出码退出。

</details>

### Agent 集成（skill）

安装内嵌的 skill 后，你的 AI agent（opencode / Claude / Cursor / Kiro）即可通过 `scafgen`
自动引导新项目 —— 它先确认需求，再非交互式运行 `scafgen new` 生成项目骨架，然后在其上继续开发。

```sh
scafgen skill install   # 然后重启 agent —— 它会自动使用 scaffold-gen skill
```

`scafgen skill <install|update|uninstall|status>` 默认安装到所有检测到的 agent（全局）。
用 `--target <agent>` 限定范围，`--local` 装到项目级，`update` 时加 `--force` 覆盖本地修改。
基于哈希的更新检测让 skill 与二进制保持同步。

## 架构设计

scaffold-gen 采用数据驱动注册表（`FrameworkSpec` + `GenKind` 枚举）进行调度。单一的
`resolve(language, framework)` 查找返回规格；编排器的 `generate(GenerationRequest)` 管线按
`GenKind`（GinSync / EmbeddedAsync / ExternalAsync / Unimplemented）分支执行。模板通过
minijinja 以自定义 `<<>>` 分隔符渲染，并在编译期内嵌于二进制中。完整设计、模板布局与变量参考见
[docs/readme/ARCHITECTURE-CN.md](ARCHITECTURE-CN.md)。

## 开发

```sh
make build      # 调试构建 → dist/scafgen
make release    # 优化的发布构建
make fmt        # 格式化（rustfmt + oxfmt 处理 YAML/JSON/Markdown）
make lint       # clippy，-D warnings
make test       # cargo test
make check      # fmt-check + lint + test（与 CI 一致）
make hooks      # 安装 pre-commit（格式化）+ pre-push（测试）钩子
make help       # 列出所有目标
```

克隆后运行一次 `make hooks` 即可启用提交时格式化、推送时测试的 git 钩子。

## 贡献

本项目使用 [Conventional Commits](https://www.conventionalcommits.org/)
（`feat:`、`fix:`、`docs:` …）—— 版本号与变更日志由
[release-please](https://github.com/googleapis/release-please) 和
[git-cliff](https://git-cliff.org) 自动生成。Fork、建分支、用规范化提交信息提交、
运行 `make check`，然后向 `main` 提 PR。

## 许可证

本项目基于 MIT 许可证 —— 详见 [LICENSE](../../LICENSE) 文件。
