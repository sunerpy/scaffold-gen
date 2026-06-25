# scaffold-gen

> Generate production-ready project scaffolds for Go, Rust, Python, and TypeScript from one interactive CLI.

[![CI](https://github.com/sunerpy/scaffold-gen/actions/workflows/ci.yml/badge.svg)](https://github.com/sunerpy/scaffold-gen/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/sunerpy/scaffold-gen)](https://github.com/sunerpy/scaffold-gen/releases)
[![Crates.io](https://img.shields.io/crates/v/scaffold-gen.svg)](https://crates.io/crates/scaffold-gen)
[![License](https://img.shields.io/crates/l/scaffold-gen.svg)](LICENSE)

[简体中文](docs/readme/README-CN.md) · English

`scafgen` is a fast, extensible scaffolding generator. Pick a language and
framework, answer a few prompts, and get a ready-to-build project with sensible
defaults, a LICENSE, and an initialized git repo.

## Table of Contents

- [Features](#features)
- [Supported Frameworks](#supported-frameworks)
- [Install](#install)
- [Quick Start](#quick-start)
- [Usage](#usage)
- [Using with an LLM](#using-with-an-llm)
- [Architecture](#architecture)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

## Features

- **Data-driven registry** — a `FrameworkSpec` table + `GenKind` enum dispatch; adding a framework
  is one registry row + a template directory, not a code change.
- **Interactive CLI** — `inquire`-driven prompts for language, framework, ports, and license.
- **Embedded templates** — everything ships inside the binary; no runtime file dependencies.
- **Environment validation** — checks required toolchains (Go ≥ 1.24, Rust ≥ 1.88, Python ≥ 3.12) before generating.
- **Structured logging** — quiet (`-q`) / verbose (`-v`) global flags; output via `tracing` to stderr.

## Supported Frameworks

| Language   | Framework  | Status                         |
| ---------- | ---------- | ------------------------------ |
| Go         | Gin        | ✅                             |
| Go         | Go-Zero    | ⚠️ not implemented             |
| Go         | MCP Server | ✅ (streamable-HTTP + SSE)     |
| Rust       | CLI App    | ✅                             |
| Rust       | Tauri      | ✅                             |
| TypeScript | Vue 3      | ✅ (offline embedded scaffold) |
| TypeScript | React      | ✅                             |
| Python     | Basic      | ✅                             |
| Python     | FastAPI    | ✅ (config-driven API)         |

## Install

**Linux / macOS** (shell):

```sh
curl -fsSL https://raw.githubusercontent.com/sunerpy/scaffold-gen/main/scripts/install.sh | sh
```

**Windows** (PowerShell):

```powershell
irm https://raw.githubusercontent.com/sunerpy/scaffold-gen/main/scripts/install.ps1 | iex
```

**Via cargo** (any platform with Rust):

```sh
cargo install scaffold-gen
```

**Prebuilt binaries** — Linux / macOS / Windows (x86_64 + aarch64) on the
[Releases](https://github.com/sunerpy/scaffold-gen/releases) page.

**From source**: `git clone … && cd scaffold-gen && make release`.

The install scripts honor `TOOL_VERSION` (pin a release) and `TOOL_INSTALL_DIR`
(custom destination) environment overrides.

## Quick Start

```sh
scafgen new my-project
```

The CLI guides you through language, framework, project configuration, and
license selection, then generates the project in `./my-project`.

## Usage

```sh
# Interactive (recommended)
scafgen new my-project

# Specify the framework directly
scafgen new my-gin-app    --framework gin
scafgen new my-gozero-app --framework go-zero
scafgen new my-tauri-app  --framework tauri
scafgen new my-vue-app    --framework vue3
scafgen new my-react-app  --framework react
scafgen new my-api        --framework fastapi --language python
scafgen new my-mcp        --framework mcp-server --language go

# Global flags (work on all subcommands)
scafgen -q new my-project   # quiet: errors only
scafgen -v new my-project   # verbose: debug output
```

Run `scafgen new --help` for the full option list.

## Using with an LLM

Install first via the [Install](#install) section, then drive the tool with these commands:

<details>
<summary>Commands an agent can drive (non-interactive)</summary>

- `scafgen new <name> --framework <fw> --language <lang>` — non-interactive generation; pass every
  choice as a flag to avoid prompts.
- `scafgen new --help` — discover available flags and frameworks.
- `scafgen --version` — print the version.

Supported frameworks: `gin`, `go-zero`, `mcp-server`, `tauri`, `vue3`, `react`, `fastapi`.

Diagnostics and errors go to stderr; the process exits non-zero on failure.

</details>

## Architecture

scaffold-gen uses a data-driven registry (`FrameworkSpec` + `GenKind` enum) to dispatch
generation. A single `resolve(language, framework)` lookup returns the spec; the
orchestrator's `generate(GenerationRequest)` pipeline branches on `GenKind` (GinSync /
EmbeddedAsync / ExternalAsync / Unimplemented). Templates are minijinja-rendered with
custom `<<>>` delimiters and embedded in the binary at compile time. See
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full design, template layout, and
variable reference.

## Development

```sh
make build      # debug build → dist/scafgen
make release    # optimized release build
make fmt        # format (rustfmt + oxfmt for YAML/JSON/Markdown)
make lint       # clippy with -D warnings
make test       # cargo test
make check      # fmt-check + lint + test (mirrors CI)
make hooks      # install pre-commit (format) + pre-push (test) hooks
make help       # list all targets
```

After cloning, run `make hooks` once to enable the format-on-commit and
test-on-push git hooks.

## Contributing

This project uses [Conventional Commits](https://www.conventionalcommits.org/)
(`feat:`, `fix:`, `docs:`, …) — version bumps and changelogs are generated
automatically by [release-please](https://github.com/googleapis/release-please)
and [git-cliff](https://git-cliff.org). Fork, branch, commit with a conventional
message, run `make check`, and open a PR against `main`.

## License

[MIT](LICENSE)
