# Scaffold-Gen

[![Crates.io](https://img.shields.io/crates/v/scaffold-gen.svg)](https://crates.io/crates/scaffold-gen)
[![License](https://img.shields.io/crates/l/scaffold-gen.svg)](LICENSE)

English | [简体中文](docs/readme/README-CN.md)

A modern, extensible scaffolding generator for creating project templates across multiple frameworks and programming languages.

## Features

- 🚀 **Modern Architecture**: Clean, modular design based on Rust traits
- 🏗️ **Three-Layer Generator Architecture**: Project, Language, and Framework level generation
- 🔌 **Unified Generator Interface**: Consistent API across all framework generators
- ⚡ **Post-Processing Pipeline**: Extensible hooks for custom project setup
- 💻 **Interactive CLI**: User-friendly prompts for project configuration
- ✅ **Environment Validation**: Automatic checking of required tools and dependencies

## Supported Frameworks

| Language | Framework | Status |
|----------|-----------|--------|
| Go | Gin | ✅ |
| Go | Go-Zero | ✅ |
| Rust | CLI App | ✅ |
| Rust | Tauri | ✅ |
| TypeScript | Vue 3 | ✅ |
| TypeScript | React | ✅ |
| Python | Basic | ✅ |

## Installation

### From crates.io

```bash
cargo install scaffold-gen
```

### From Source

```bash
git clone https://github.com/sunerpy/scaffold-gen.git
cd scaffold-gen
make release
```

### Pre-built Binaries

Download pre-built binaries from the [Releases](https://github.com/sunerpy/scaffold-gen/releases) page.

## Quick Start

### Interactive Mode (Recommended)

```bash
scafgen new my-project
```

The CLI will guide you through:

- Language selection (Go, Rust, TypeScript, Python)
- Framework selection (Gin, Go-Zero, Tauri, Vue3, React, etc.)
- Project configuration (host, port, features)
- License selection

### Direct Framework Specification

```bash
# Create a Gin project
scafgen new my-gin-app --framework gin

# Create a Go-Zero project
scafgen new my-gozero-app --framework go-zero

# Create a Tauri project
scafgen new my-tauri-app --framework tauri

# Create a Vue3 project
scafgen new my-vue-app --framework vue3

# Create a React project
scafgen new my-react-app --framework react
```

## Architecture

### Three-Layer Generator Architecture

```
┌─────────────────────────────────────────┐
│           GeneratorOrchestrator         │
│      (Coordinates all generators)       │
└─────────────────┬───────────────────────┘
                  │
    ┌─────────────┼─────────────┐
    ▼             ▼             ▼
┌────────┐  ┌──────────┐  ┌───────────┐
│ Project │  │ Language │  │ Framework │
│Generator│  │Generator │  │ Generator │
└────────┘  └──────────┘  └───────────┘
    │             │             │
    ▼             ▼             ▼
 LICENSE      Go/Rust/      Gin/Tauri/
 Git/README   Python/TS     Vue3/React
```

#### 1. Project Generator

Handles common project files:

- LICENSE file generation
- Git repository initialization
- Pre-commit hooks installation
- README file generation

#### 2. Language Generator

Sets up language-specific environment:

- **GoGenerator**: Go module initialization, dependency management
- **RustGenerator**: Cargo project initialization
- **PythonGenerator**: Python project structure
- **TypeScriptGenerator**: Node.js/npm configuration

#### 3. Framework Generator

Generates framework-specific code structure:

- **GinGenerator**: Gin web framework project structure
- **GoZeroGenerator**: Go-Zero microservice framework structure
- **TauriGenerator**: Tauri desktop application structure
- **Vue3Generator**: Vue 3 frontend project structure
- **ReactGenerator**: React frontend project structure

## Template System

The generator uses a hierarchical template system:

```
templates/
├── frameworks/          # Framework-specific templates
│   ├── go/
│   │   ├── gin/        # Gin framework templates
│   │   └── go-zero/    # Go-Zero framework templates
│   ├── rust/
│   │   └── tauri/      # Tauri framework templates
│   └── typescript/
│       ├── vue3/       # Vue 3 framework templates
│       └── react/      # React framework templates
├── languages/          # Language-specific templates
│   ├── go/
│   ├── rust/
│   ├── python/
│   └── typescript/
└── licenses/           # License templates
    ├── MIT.tmpl
    ├── Apache-2.0.tmpl
    └── GPL-3.0.tmpl
```

### Template Variables

#### Common Variables

- `{{project_name}}` - Project name
- `{{author}}` - Project author
- `{{license}}` - License type
- `{{year}}` - Current year

#### Framework-Specific Variables

- `{{host}}` - Server host (default: localhost)
- `{{port}}` - HTTP port (default: 8080)
- `{{grpc_port}}` - gRPC port (Go-Zero specific)
- `{{enable_swagger}}` - Enable Swagger documentation
- `{{enable_database}}` - Enable database support

## Development

### Build Commands

```bash
# Debug build
make build

# Release build
make release

# Run tests
make test

# Run linter
make lint

# Format code
make fmt

# Run all CI checks
make ci
```

### Project Structure

```
src/
├── commands/           # CLI command implementations
├── generators/         # Generator modules
│   ├── core/          # Core generator traits and utilities
│   ├── project/       # Project-level generator
│   ├── language/      # Language-level generators
│   ├── framework/     # Framework-level generators
│   └── orchestrator.rs # Generator orchestrator
├── scaffold.rs        # Core scaffolding system
├── template_engine.rs # Template processing engine
└── utils/             # Utility modules
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
