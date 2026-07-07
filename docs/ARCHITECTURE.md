# Architecture

[简体中文](readme/ARCHITECTURE-CN.md) · English

scaffold-gen uses a three-layer generator architecture and a hierarchical
template system. This document covers both in detail.

## Three-Layer Generator Architecture

```
┌─────────────────────────────────────────┐
│           GeneratorOrchestrator          │
│       (Coordinates all generators)       │
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

### 1. Project Generator

Handles common project files:

- LICENSE file generation
- Git repository initialization
- Pre-commit hooks installation
- README file generation

### 2. Language Generator

Sets up the language-specific environment:

- **GoGenerator** — Go module initialization, dependency management
- **RustGenerator** — Cargo project initialization
- **PythonGenerator** — Python project structure
- **TypeScriptGenerator** — Node.js/npm configuration

### 3. Framework Generator

Generates framework-specific code structure:

- **GinGenerator** — Gin web framework project structure
- **TauriGenerator** — Tauri desktop application structure

Vue 3 and React are embedded-template frameworks (`GenKind::EmbeddedAsync`) with no
dedicated generator struct; they are rendered offline by
`orchestrator::generate_vue3_embedded` / `generate_react_embedded` from
`templates/frameworks/typescript/{vue3,react}/`.

Go-Zero resolves to `GenKind::Unimplemented`; the enum variant is kept only so the
CLI can emit a clear error, and it has no generator struct.

The generation order is Framework → Language → Project, coordinated by the
orchestrator in [`src/generators/orchestrator.rs`](../src/generators/orchestrator.rs).

## Template System

The generator uses a hierarchical template system. Templates are embedded into
the binary at build time via the `include_dir` crate, so no external files are
required at runtime.

```
templates/
├── frameworks/          # Framework-specific templates
│   ├── go/
│   │   ├── gin/         # Gin framework templates
│   │   └── go-zero/     # Go-Zero framework templates
│   ├── rust/
│   │   └── tauri/       # Tauri framework templates
│   └── typescript/
│       ├── vue3/        # Vue 3 framework templates
│       └── react/       # React framework templates
├── languages/           # Language-specific templates
│   ├── go/
│   ├── rust/
│   ├── python/
│   └── typescript/
└── licenses/            # License templates
    ├── MIT.tmpl
    ├── Apache-2.0.tmpl
    └── GPL-3.0.tmpl
```

Files ending in `.tmpl` are rendered through Handlebars and the suffix is
stripped; all other files are copied verbatim.

### Template Variables

#### Common Variables

| Variable           | Description    |
| ------------------ | -------------- |
| `{{project_name}}` | Project name   |
| `{{author}}`       | Project author |
| `{{license}}`      | License type   |
| `{{year}}`         | Current year   |

#### Framework-Specific Variables

| Variable              | Description                      |
| --------------------- | -------------------------------- |
| `{{host}}`            | Server host (default: localhost) |
| `{{port}}`            | HTTP port (default: 8080)        |
| `{{grpc_port}}`       | gRPC port (Go-Zero specific)     |
| `{{enable_swagger}}`  | Enable Swagger documentation     |
| `{{enable_database}}` | Enable database support          |

## Source Layout

```
src/
├── commands/            # CLI command implementations
├── generators/          # Generator modules
│   ├── core/            # Core generator traits and utilities
│   ├── project/         # Project-level generator
│   ├── language/        # Language-level generators
│   ├── framework/       # Framework-level generators
│   └── orchestrator.rs  # Generator orchestrator
├── scaffold.rs          # Core scaffolding system
├── template_engine.rs   # Template processing engine
└── utils/               # Utility modules
```
