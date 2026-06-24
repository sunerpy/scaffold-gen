# 架构设计

简体中文 · [English](../ARCHITECTURE.md)

scaffold-gen 采用三层生成器架构与分层模板系统。本文档详细说明两者。

## 三层生成器架构

```
┌─────────────────────────────────────────┐
│           GeneratorOrchestrator          │
│         (协调所有生成器的执行)            │
└─────────────────┬────────────────────────┘
                  │
    ┌─────────────┼─────────────┐
    ▼             ▼             ▼
┌─────────┐  ┌──────────┐  ┌───────────┐
│ Project │  │ Language │  │ Framework │
│Generator│  │Generator │  │ Generator │
└─────────┘  └──────────┘  └───────────┘
    │             │             │
    ▼             ▼             ▼
 LICENSE      Go/Rust/      Gin/Tauri/
 Git/README   Python/TS     Vue3/React
```

### 1. 项目级生成器 (ProjectGenerator)

负责通用项目文件的生成：

- LICENSE 文件生成
- Git 仓库初始化
- Pre-commit hooks 安装
- README 文件生成

### 2. 语言级生成器 (LanguageGenerator)

处理特定编程语言的设置：

- **GoGenerator** — Go 模块初始化、依赖管理
- **RustGenerator** — Cargo 项目初始化
- **PythonGenerator** — Python 项目结构
- **TypeScriptGenerator** — Node.js/npm 配置

### 3. 框架级生成器 (FrameworkGenerator)

生成框架特定的代码结构：

- **GinGenerator** — Gin web 框架项目结构
- **GoZeroGenerator** — Go-Zero 微服务框架结构
- **TauriGenerator** — Tauri 桌面应用结构
- **Vue3Generator** — Vue 3 前端项目结构
- **ReactGenerator** — React 前端项目结构

生成顺序为 Framework → Language → Project，由
[`src/generators/orchestrator.rs`](../../src/generators/orchestrator.rs) 中的编排器协调。

## 模板系统

生成器使用分层模板系统。模板在构建时通过 `include_dir` crate 嵌入二进制，
因此运行时无需任何外部文件。

```
templates/
├── frameworks/          # 框架特定模板
│   ├── go/
│   │   ├── gin/         # Gin 框架模板
│   │   └── go-zero/     # Go-Zero 框架模板
│   ├── rust/
│   │   └── tauri/       # Tauri 框架模板
│   └── typescript/
│       ├── vue3/        # Vue 3 框架模板
│       └── react/       # React 框架模板
├── languages/           # 语言特定模板
│   ├── go/
│   ├── rust/
│   ├── python/
│   └── typescript/
└── licenses/            # 许可证模板
    ├── MIT.tmpl
    ├── Apache-2.0.tmpl
    └── GPL-3.0.tmpl
```

以 `.tmpl` 结尾的文件会经过 Handlebars 渲染并去除后缀；其他文件原样复制。

### 模板变量

#### 通用变量

| 变量               | 说明       |
| ------------------ | ---------- |
| `{{project_name}}` | 项目名称   |
| `{{author}}`       | 项目作者   |
| `{{license}}`      | 许可证类型 |
| `{{year}}`         | 当前年份   |

#### 框架特定变量

| 变量                  | 说明                          |
| --------------------- | ----------------------------- |
| `{{host}}`            | 服务器主机（默认: localhost） |
| `{{port}}`            | HTTP 端口（默认: 8080）       |
| `{{grpc_port}}`       | gRPC 端口（Go-Zero 专用）     |
| `{{enable_swagger}}`  | 是否启用 Swagger 文档         |
| `{{enable_database}}` | 是否启用数据库支持            |

## 源码结构

```
src/
├── commands/            # CLI 命令实现
├── generators/          # 生成器模块
│   ├── core/            # 核心生成器 traits 和工具
│   ├── project/         # 项目级生成器
│   ├── language/        # 语言级生成器
│   ├── framework/       # 框架级生成器
│   └── orchestrator.rs  # 生成器编排器
├── scaffold.rs          # 核心脚手架系统
├── template_engine.rs   # 模板处理引擎
└── utils/               # 工具模块
```
