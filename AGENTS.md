# SCAFFOLD-GEN KNOWLEDGE BASE

**Version:** v0.7.0 **Updated:** 2026-06-25 **Branch:** main

## OVERVIEW

Rust CLI scaffolding tool (`scafgen`) generating project templates for Go/Rust/Python/TypeScript
via a data-driven registry dispatch. A `FrameworkSpec` registry + `GenKind` enum replace the old
78-line match tree; the orchestrator's `generate(GenerationRequest)` pipeline branches on `GenKind`.
6 subcommands: `new`, `list`, `version`, `completions`, `self-update`, `skill`.

## STRUCTURE

```
scaffold-gen/
├── src/
│   ├── main.rs              # CLI entry (clap) — Commands enum (6 variants), SkillAction, async fn run()
│   ├── lib.rs               # Library exports
│   ├── logging.rs           # tracing init, Verbosity enum, RUST_LOG override
│   ├── commands/
│   │   ├── new.rs           # NewCommand + builders, execute(), generate_project()
│   │   ├── prompts.rs       # All inquire Select/Text/Confirm prompt methods (pub(super))
│   │   ├── env_check.rs     # Per-language environment pre-check (pub(super))
│   │   ├── list.rs          # scafgen list [--json] — driven by registry::all_specs()
│   │   ├── version.rs       # scafgen version — prints name/version/repo to stdout
│   │   ├── completions.rs   # scafgen completions <shell> [--install] (clap_complete)
│   │   ├── self_update.rs   # scafgen self-update [--check] [--force] [--tag] (self_update+rustls)
│   │   └── skill.rs         # scafgen skill <install|update|uninstall|status> handler
│   ├── skill/               # Skill installer engine (embed/engine/targets)
│   │   ├── mod.rs           # AgentId, Location, InstallContext, public API re-exports
│   │   ├── embed.rs         # include_str! SKILL.md + git-blob-sha1 hash + sidecar marker
│   │   ├── engine.rs        # write/update/uninstall/status to a skill dir
│   │   └── targets.rs       # per-agent (opencode/claude/cursor/kiro) global+local dirs
│   ├── generators/
│   │   ├── registry.rs      # FrameworkSpec, GenKind enum, resolve(), all_specs()
│   │   ├── orchestrator.rs  # GeneratorOrchestrator, GenerationRequest, generate() pipeline,
│   │   │                    #   render_build_tooling(), build_tooling_context() — ~665 LOC
│   │   ├── external.rs      # ExternalAsync path: Tauri / React (Vue3 moved to embedded)
│   │   ├── gin_options.rs   # GinProjectOptions struct + builders
│   │   ├── core/            # Generator/ProjectGenerator traits, BaseParams, TemplateProcessor,
│   │   │                    #   context.rs (build_base_context 31 keys), validation.rs
│   │   ├── language/        # go/, rust/, python/
│   │   ├── framework/       # gin/, tauri/, vue3/, react/ (no go_zero/ — deleted Phase 2)
│   │   └── project/         # ProjectScaffolder — LICENSE, git init, pre-commit
│   ├── template_engine.rs   # minijinja (custom <<>> delimiters) + embedded templates
│   ├── constants.rs         # Language/Framework enums, frameworks_for_language(),
│   │                        #   Language::build_dir(), string_utils
│   └── utils/
│       ├── env_checker.rs   # Async toolchain version checks (uv/rust/python)
│       ├── toolchain.rs     # tool_available() + ExternalCommand builder
│       └── go_tools.rs
├── templates/               # Embedded .tmpl files (include_dir!)
│   ├── build/               # --with-build templates: go/ rust/ python/ typescript/
│   │                        #   each has Makefile.tmpl + Dockerfile.tmpl
│   ├── frameworks/          # go/{gin,go-zero,mcp-server}/ rust/{tauri}/ python/{fastapi,mcp-python}/
│   │                        #   typescript/{vue3,react}/
│   ├── languages/           # go/, rust/, python/ (pure-language paths)
│   └── licenses/            # MIT, Apache-2.0, GPL-3.0, etc.
├── skills/
│   └── scaffold-gen/
│       ├── SKILL.md         # Embedded agent skill (include_str! into src/skill/embed.rs)
│       └── evals/evals.json # Eval test prompts (skill-creator loop artifact)
├── tests/
│   └── generation.rs        # Integration tests (public API, 8 tests)
├── Makefile                 # Primary task runner
└── Cargo.toml               # Edition 2024, binary: scafgen, version: 0.7.0
```

## FRAMEWORK REGISTRY

9 frameworks total. `frameworks_for_language` in `constants.rs`:

| Language   | Frameworks                     |
| ---------- | ------------------------------ |
| Go         | Gin, GoZero, McpServer         |
| Python     | None, FastApi, McpServerPython |
| Rust       | None, Tauri                    |
| TypeScript | Vue3, React                    |

GenKind dispatch (`registry.rs` REGISTRY):

| Framework       | Language   | GenKind       | Notes                                                                          |
| --------------- | ---------- | ------------- | ------------------------------------------------------------------------------ |
| Gin             | Go         | GinSync       | Reads GinProjectOptions (host/port/swagger/...)                                |
| GoZero          | Go         | Unimplemented | Returns clear error; no generator struct                                       |
| McpServer       | Go         | EmbeddedAsync | Gin + go-sdk, streamable-HTTP + SSE, buf/proto                                 |
| FastApi         | Python     | EmbeddedAsync | config.toml-driven; uvicorn reload-loop fix                                    |
| McpServerPython | Python     | EmbeddedAsync | FastMCP/official backend, streamable `/mcp` + SSE `/sse`, Pydantic auto-schema |
| None            | Python     | EmbeddedAsync | uv init + structlog                                                            |
| None            | Rust       | EmbeddedAsync | cargo init + proto/error-gen opts                                              |
| Tauri           | Rust       | ExternalAsync | pnpm create-tauri-app shell-out                                                |
| Vue3            | TypeScript | EmbeddedAsync | Full Vite+Vue3+TS+Tailwind, .env-driven                                        |
| React           | TypeScript | ExternalAsync | pnpm create vite shell-out                                                     |

Go/TypeScript + `Framework::None` → no pure-language path (error: "language requires a framework").

## WHERE TO LOOK

| Task                       | Location                                      | Notes                                                  |
| -------------------------- | --------------------------------------------- | ------------------------------------------------------ |
| Add new framework          | `constants.rs` + `generators/registry.rs`     | See generators/AGENTS.md                               |
| Add CLI subcommand         | `src/main.rs` Commands enum + `src/commands/` | New module + wire in `run()` match arm                 |
| Add flag to `new`          | `src/main.rs` Commands::New + `new.rs`        | Interactive prompts in `prompts.rs`                    |
| `list` output              | `src/commands/list.rs`                        | Driven by `registry::all_specs()` — no manual list     |
| `self-update` behavior     | `src/commands/self_update.rs`                 | Blocking work runs on a thread (not in tokio)          |
| `completions` output       | `src/commands/completions.rs`                 | Machine output → stdout; hints → tracing stderr        |
| Modify template rendering  | `src/template_engine.rs`                      | minijinja custom `<<>>` delimiters                     |
| Add template helper/filter | `src/template_engine.rs`                      | `register_helper()` / `register_filter()`              |
| Check tool availability    | `src/utils/toolchain.rs`                      | `tool_available()` for PATH probes                     |
| Async version checks       | `src/utils/env_checker.rs`                    | uv/Rust/Python version reads                           |
| Generation pipeline order  | `src/generators/orchestrator.rs`              | `generate()` → kind dispatch → `render_build_tooling`  |
| `--with-build` step        | `orchestrator.rs::render_build_tooling`       | Runs after framework/language; `Language::build_dir()` |
| Build template content     | `templates/build/<lang>/`                     | Makefile.tmpl + Dockerfile.tmpl per language           |
| External shell-out logic   | `src/generators/external.rs`                  | Tauri / React only (Vue3 is now embedded)              |
| Manage agent skill install | `src/skill/` + `src/commands/skill.rs`        | Embedded `skills/scaffold-gen/SKILL.md`; hash update   |
| Output verbosity / logging | `src/logging.rs`                              | Verbosity enum, tracing init                           |
| Framework-language mapping | `constants.rs::frameworks_for_language()`     | Single source of truth for the `list` command          |

## CLI SUBCOMMANDS

```
scafgen new <name>         # Interactive or flag-driven project generation
scafgen list [--json]      # Show all frameworks + availability status
scafgen version            # Print name, version, repository URL (stdout)
scafgen completions <shell> [--install]  # bash/zsh/fish/powershell/elvish
scafgen self-update [--check] [--force] [--tag <tag>]
scafgen skill <install|update|uninstall|status>  # install embedded SKILL.md into agent dirs
```

Global flags (all subcommands): `-q/--quiet` (errors only), `-v/--verbose` (debug).

`new` flags: `--framework`, `--language`, `--host`, `--port`, `--license`, `--precommit`,
`--swagger`, `--proto-gen`, `--error-gen`, `--with-build`, `--mcp-backend <fastmcp|official>`.

`skill` flags: `--target <agent>` (opencode/claude/cursor/kiro; repeatable; default all detected),
`--global` (default) / `--local`, `-y/--yes` (install/uninstall), `--force` (update).

Machine-consumable output (completions script, version string, list table/JSON) → **stdout**.
All tracing/diagnostics → **stderr**.

## CONVENTIONS

### Rust Style

- **Edition 2024** — use latest features
- **Interpolated strings**: `format!("{variable}")` NOT `format!("{}", variable)`
- **Line width**: 100 chars (`.rustfmt.toml`)
- **Params layout**: `fn_params_layout = "Tall"`

### CLI

- Always use `clap::ColorChoice::Auto`
- Interactive prompts via `inquire` crate
- Global `-q/--quiet` (errors only) and `-v/--verbose` (debug) flags on top-level `Cli`
- All tracing output goes to **stderr**; stdout stays clean for machine output (`list`, `version`,
  `completions`)
- `self-update` blocking work runs on a dedicated OS thread via `std::thread::scope` — never call
  the blocking `self_update` crate from inside the tokio runtime

### Template System

- Engine: **minijinja** with custom delimiters: `<<var>>`, `<%if cond%>` / `<%endif%>`, `<#comment#>`
- Files ending `.tmpl` → rendered + suffix stripped
- Non-`.tmpl` files → copied verbatim
- Templates embedded via `include_dir!` crate — no runtime file I/O
- `UndefinedBehavior::Lenient`: undefined vars render to empty string (no panic)

## ANTI-PATTERNS (THIS PROJECT)

| Pattern                                  | Why Forbidden                                 |
| ---------------------------------------- | --------------------------------------------- |
| `format!("{}", var)`                     | Use `format!("{var}")` per `.clinerules`      |
| `.unwrap()` on regex captures            | Panics on format changes — use `context()`    |
| `Default::default().expect()`            | Prefer proper error handling in constructors  |
| Hardcoded version fallbacks              | Move to `constants.rs` or make configurable   |
| `#[allow(dead_code)]` for future methods | Delete dead code; don't reserve it            |
| Calling blocking `self_update` in async  | Must use `std::thread::scope` to escape tokio |

## GOTCHAS

1. **Magic suffix stripping**: `.tmpl` ALWAYS stripped — cannot generate `.tmpl` files
2. **Strict minimum versions**: Go ≥1.24, Rust ≥1.88, Python ≥3.12 enforced
3. **Auto-install side effects**: Tauri generator installs `create-tauri-app` via pnpm without
   confirmation
4. **Go-Zero unimplemented**: `Framework::GoZero` resolves to `GenKind::Unimplemented` — returns a
   clear error; no generator struct exists; enum variant kept for CLI discoverability
5. **Tests: 169 total**: 50 lib + 110 bin inline + 9 integration in `tests/generation.rs`; `make
test` covers all
6. **Vue3 is EmbeddedAsync**: moved from ExternalAsync — full offline scaffold, optional `pnpm
install` post-step; `external.rs` no longer contains Vue3 logic
7. **`--with-build` is opt-in interactive**: omitting it on non-TTY stdin triggers an `inquire`
   Confirm error (same as `--precommit`); scripts must pass `--with-build true|false` explicitly
8. **FastAPI reload-loop fix**: `uvicorn.run` in the generated `main.py` uses
   `reload_dirs=["app"]` + `reload_excludes` to prevent log writes inside the watched dir from
   triggering infinite reloads
9. **orchestrator.rs is ~665 LOC**: exceeds the 250-line guideline; known refactor candidate,
   but not yet split (splitting the tightly-coupled dispatch would fragment logic)
10. **AGENTS.md is now tracked**: removed from `.gitignore`; oxfmt formats it — `make fmt-check`
    enforces correct Markdown formatting on these files
11. **mcp-python backend selection**: `McpServerPython` supports `--mcp-backend fastmcp|official`
    (default `fastmcp`). Dual transport is always-on: `/mcp` (streamable-HTTP) + `/sse` (SSE);
    `sse_enabled` in config.toml toggles the SSE mount at runtime. `make test` runs pytest with an
    in-memory client (no live server needed) — backend switch changes only `app/server.py`.
12. **include_dir! stale cache**: templates embed via `include_dir!` at compile time. Adding NEW
    template files does not always trigger re-embed on incremental `cargo build` — the rendered
    project silently misses new files. Fix: `touch build.rs` (or `cargo clean`) to bust the cache
    before rendering/testing newly-added templates.

## COMMANDS

```bash
# Development
make build          # Debug build → dist/scafgen
make release        # Optimized release build
make release-upx    # Release + UPX compression

# Quality
make fmt            # Format code (rustfmt + oxfmt for YAML/JSON/Markdown)
make fmt-check      # Check formatting (CI gate — also checks AGENTS.md now)
make lint           # Clippy with -D warnings
make test           # cargo test (50 lib + 110 bin + 9 integration)
make ci             # fmt-check + lint + test

# Cross-compile
make release-target TARGET=x86_64-unknown-linux-musl

# Install
make install        # → ~/.cargo/bin/scafgen
make hooks          # Install pre-commit (fmt) + pre-push (test) git hooks
```

## RELEASE PROFILE

```toml
# Cargo.toml [profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
```

## CI/CD

- **CI**: `.github/workflows/ci.yml` — fmt-check, lint, test on push/PR
- **Release**: `.github/workflows/release.yml` — multi-platform builds on `v*` tags (6 targets:
  x86_64/aarch64 × linux-musl / apple-darwin / pc-windows-msvc), publishes to crates.io

## NOTES

- Binary name `scafgen` (not `scaffold-gen`); crate name `scaffold-gen`
- `build.rs` copies binary to `dist/` for convenience
- Pre-commit hooks via `.pre-commit-config.yaml` (commitizen, clippy)
- Clippy ignores: `unused`, `clippy::uninlined_format_args`
- AGENTS.md + src/generators/AGENTS.md are **tracked + committed** (removed from `.gitignore`);
  oxfmt formats them as Markdown — `make fmt-check` will fail if they have formatting drift
- `scafgen list` output is entirely data-driven from `registry::all_specs()` — adding a REGISTRY
  row automatically updates the list; no separate listing to maintain
