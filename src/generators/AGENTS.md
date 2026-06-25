# GENERATORS MODULE

**Updated:** 2026-06-25 (v0.5.0)

Data-driven dispatch: a `FrameworkSpec` registry + `GenKind` enum replace the old
78-line `match` tree. Adding a framework is a data change, not a code change.

## ARCHITECTURE

```
generators/
├── registry.rs          # FrameworkSpec + GenKind enum + resolve() + all_specs()
├── orchestrator.rs      # GeneratorOrchestrator + GenerationRequest + generate() pipeline
│                        #   + render_build_tooling() + build_tooling_context() — ~665 LOC
├── external.rs          # ExternalAsync path: Tauri / React (pub(super) impl block)
├── gin_options.rs       # GinProjectOptions struct + builders
├── core/
│   ├── generator.rs     # Generator trait + ProjectGenerator trait
│   ├── base_parameters.rs   # BaseParams struct + InheritableParams trait
│   ├── context.rs       # build_base_context() — 31 keys + aliases
│   ├── parameters.rs    # Parameters trait
│   ├── template_processor.rs
│   └── validation.rs
├── project/             # ProjectScaffolder — LICENSE, git init, pre-commit
├── language/            # go/, rust/, python/
└── framework/           # gin/, tauri/, vue3/, react/ (no go_zero/ — deleted Phase 2)
```

Note: `LanguageGenerator` and `FrameworkGenerator` traits were deleted in Phase 2 (zero live
callers). Language and framework generators now only `impl Generator`.

## DISPATCH FLOW

```
new.rs::generate_project()
  └─ registry::resolve(language, framework) -> FrameworkSpec
       └─ orchestrator.generate(GenerationRequest { spec, ... }).await
            └─ match spec.kind
                 GinSync       -> generate_gin_project()     (sync, reads GinProjectOptions)
                 EmbeddedAsync -> generate_embedded()        (Python/Rust/Vue3/McpServer)
                 ExternalAsync -> generate_external()        (Tauri / React pnpm shell-out)
                 Unimplemented -> Err("GoZero 项目生成尚未实现")
            └─ if enable_build: render_build_tooling()       (--with-build post-step)
```

`resolve` is the single lookup point. `Framework::None` dispatches via `pure_language_spec`
(Python/Rust → EmbeddedAsync; Go/TS+None → None, drives "language requires a framework" error).

## GENKIND → FRAMEWORK MAPPING

| GenKind       | Frameworks                             | Notes                                                |
| ------------- | -------------------------------------- | ---------------------------------------------------- |
| GinSync       | Gin                                    | Sync; reads `GinProjectOptions` for all options      |
| EmbeddedAsync | McpServer, FastApi, None(Python/Rust), | Fully offline; templates embedded in binary          |
|               | Vue3                                   | Vue3: optional `pnpm install` post-step              |
| ExternalAsync | Tauri, React                           | Shell-out to pnpm/create-tauri-app; in `external.rs` |
| Unimplemented | GoZero                                 | Returns error; enum variant kept for discoverability |

## EXECUTION ORDER (EmbeddedAsync)

1. **Framework/Language** — renders templates or runs toolchain (`uv init`, `cargo init`, etc.)
2. **Project** — LICENSE, `.gitignore`, `git init`, pre-commit (`run_project_step`)
3. **Build tooling** — if `enable_build`: `render_build_tooling` renders `templates/build/<lang>/`

External frameworks (Tauri/React) handle their own sequence inside `generate_tauri/react_project`
in `external.rs`, then call `run_project_step` for the project tail.

GinSync has its own full sequence in `generate_gin_project`: framework → go mod → project →
`gin_generator.post_process` (swag init, go mod tidy).

## THE `--WITH-BUILD` STEP

`render_build_tooling(&GenerationRequest)` in `orchestrator.rs`:

- Runs AFTER the kind dispatch, for ALL kinds (GinSync/EmbeddedAsync/ExternalAsync).
- Maps `request.spec.language` → `build/<dir>` via `Language::build_dir()` (constants.rs).
- Renders `templates/build/<lang>/Makefile.tmpl` + `Dockerfile.tmpl` into project root.
- If template dir missing: `tracing::warn!` + skip (non-fatal). Don't create an empty dir.
- `build_tooling_context`: per-language context — Go→GoParams (module_name/go_version/host/port),
  Python→PythonParams (python_version/package_name/host/port), Rust→RustParams, TS→ProjectParams.
- Old per-framework Makefile/Dockerfile templates were removed; build tooling ONLY from
  `--with-build` (mcp-server keeps its own `Makefile.tmpl` for `buf generate` — framework-specific).

## WHERE TO LOOK

| Task                       | Location                                     | Notes                                          |
| -------------------------- | -------------------------------------------- | ---------------------------------------------- |
| Add/remove a framework     | `constants.rs` enum + `registry.rs` REGISTRY | See "Adding a framework" below                 |
| Change dispatch strategy   | `registry.rs` `GenKind` + orchestrator       |                                                |
| Add CLI flag to `new`      | `src/main.rs` + `src/commands/new.rs`        | Prompts in `prompts.rs`                        |
| Modify template rendering  | `src/template_engine.rs`                     | minijinja custom `<<>>` delimiters             |
| Add template helper/filter | `src/template_engine.rs`                     | `register_helper()` / `register_filter()`      |
| Check tool availability    | `src/utils/toolchain.rs`                     | `tool_available()` for PATH probes             |
| Adjust embedded generation | `orchestrator.rs::generate_embedded`         | Branch by language/framework                   |
| External shell-out logic   | `external.rs`                                | Tauri / React pnpm bodies only (Vue3 NOT here) |
| Gin-specific options       | `gin_options.rs`                             | `GinProjectOptions` struct                     |
| Template context keys      | `core/context.rs`                            | `build_base_context()` — 31 keys + aliases     |
| Prompts / interactive flow | `src/commands/prompts.rs`                    | All `inquire` Select/Text/Confirm calls        |
| Environment pre-check      | `src/commands/env_check.rs`                  | Per-language Git/Go/uv/Cargo/Node/pnpm probes  |
| `--with-build` rendering   | `orchestrator.rs::render_build_tooling`      | `Language::build_dir()` → `build/<lang>/`      |
| Build template content     | `templates/build/<lang>/`                    | Makefile.tmpl + Dockerfile.tmpl per language   |

## TRAIT HIERARCHY (current)

```rust
Generator (base — core/generator.rs)
  fn name() / get_template_path() / generate() / render_embedded_templates()

ProjectGenerator (project-level — core/generator.rs)
  fn generate_license() / init_git_repository() / install_precommit()
  impl'd by ProjectScaffolder (project/generator.rs)
```

`LanguageGenerator` and `FrameworkGenerator` traits were deleted in Phase 2. No impl blocks
for them exist anywhere. Language and framework generators only `impl Generator`.

## ADDING A NEW FRAMEWORK

### Embedded-template case (e.g. a new Python framework)

1. **`constants.rs`**: add `Framework::NewFw` variant; add arms to `as_str` / `display_name` /
   `parse_from_str`; add to `frameworks_for_language(Python)`.
2. **`registry.rs`**: add one `FrameworkSpec` row to `REGISTRY`:

   ```rust
   FrameworkSpec {
       framework: Framework::NewFw,
       language: Language::Python,
       kind: GenKind::EmbeddedAsync,
       description_template: "A NewFw service: {name}",
       accepts_proto_error_gen: false,
       next_steps: &["  uv run python main.py  # Start dev server"],
   },
   ```

3. **`templates/frameworks/python/new-fw/`**: drop the `.tmpl` files here.
4. **Orchestrator**: if the new framework needs a step beyond the generic Python path, add a
   `Language::Python if spec.framework == Framework::NewFw` branch in `generate_embedded` and
   write a thin `generate_new_fw_language(...)` method (mirrors `generate_fastapi_language`).
   For a pure-template framework that reuses the Python language step, nothing else is needed.

No new `generate_<name>_project()` method. No new match arm in `new.rs`.
The registry test `every_listed_framework_resolves` auto-covers the new REGISTRY entry.

### External scaffolder case (pnpm-based tool)

Same steps 1-2 with `kind: GenKind::ExternalAsync`, then:

3. Add templates if any.
4. Write `generate_<name>_project(&GenerationRequest)` in `external.rs` as
   `impl GeneratorOrchestrator`.
5. Add `Framework::Name =>` arm in `generate_external` in `orchestrator.rs`.

Description and next-steps still come from REGISTRY data.

## PARAMETER COMPOSITION

Framework params embed:

- `BaseParams` (project_name, output_path, description, author, license, …)
- `InheritableParams` blanket impl provides `Parameters::to_template_context` via
  `build_base_context()` in `core/context.rs` (31 keys + aliases).
- Framework-specific fields (host, port, enable_swagger, …) set on `base.host`/`base.port`
  or via the generator's own context extension.

## TEMPLATE SYSTEM

- Engine: **minijinja** with custom delimiters: `<<var>>`, `<%if cond%>` /
  `<%endif%>`, `<#comment#>`.
- Files ending `.tmpl` → rendered + suffix stripped.
- Non-`.tmpl` files → copied verbatim.
- Templates embedded at compile time via `include_dir!` — no runtime file I/O.
- Custom filters: `to_camel_case`, `to_snake_case` (hyphen-splitting only — different from
  `constants::string_utils::to_snake_case` which also splits camelCase).
- `UndefinedBehavior::Lenient`: undefined vars render to empty string — no panics.

## CONVENTIONS

- Each generator module: `mod.rs` + `generator.rs` + `parameters.rs`
- Async for generators that call external tools; sync for pure-template generators
- `post_process()` for shell commands after file generation (Gin only currently)
- `format!("{var}")` NOT `format!("{}", var)` — project-wide rule
- No `#[allow(dead_code)]` for future-reserved methods — delete dead code

## PYTHON TEMPLATES — STRUCTLOG

Both `templates/languages/python/` (basic) and `templates/frameworks/python/fastapi/` use
structlog. Generated projects get:

- `loggers/logger.py` or `app/logging.py` — `init_logging(config)` with RotatingFileHandler
  (optional gzip), console/JSON format switch, noisy-logger suppression.
- Per-env config TOML (`[log]` section): dev=debug/console, test=warning/json, prod=info/json+compress.
- FastAPI: `pydantic-settings` `LogConfig` model on `Settings`; `init_logging(settings.log)` in
  `main.py` lifespan.

## FASTAPI GOTCHA — UVICORN RELOAD LOOP

Generated `main.py` uses `uvicorn.run(..., reload_dirs=["app"], reload_excludes=["logs", ...])`.
Without scoping, the structlog file handler writing to `logs/` inside the project root triggers
WatchFiles on every write → infinite reload loop. The fix scopes watching to `app/` only.

## GO-ZERO STATUS

`Framework::GoZero` resolves to `GenKind::Unimplemented`. `orchestrator.generate()` returns
`Err("GoZero 项目生成尚未实现")`. No GoZero generator struct, parameters, or templates exist.
Enum variant kept so the CLI surfaces a clear error rather than silently ignoring the input.

## VUE3 — EMBEDDED (.ENV-DRIVEN)

Vue3 is `GenKind::EmbeddedAsync` (moved from ExternalAsync). `generate_vue3_embedded` in
`orchestrator.rs` renders `templates/frameworks/typescript/vue3/`, then attempts `pnpm install`
if pnpm is on PATH (non-fatal warn on failure). Generated project is `.env`-driven via Vite's
`loadEnv`: `VITE_DEV_HOST`, `VITE_DEV_PORT`, `VITE_DEV_ALLOWED_HOSTS`, `VITE_API_BASE_URL`,
`VITE_DEV_PROXY_TARGET`. `external.rs` contains NO Vue3 logic.

## MCP SERVER SCAFFOLD (GO)

`Framework::McpServer` (Go, EmbeddedAsync). Generates:

- Dual transport: `/mcp` + `/mcp/` (streamable HTTP) and `/sse` + `/sse/` (SSE, 2024-11-05 spec).
- `config.toml`-driven addr/port/mcp_path/sse_path/log.
- `proto/echo.proto` → `make generate` (buf) → `GetJSONSchemaBytes()` wired as `InputSchema`.
- Layout: `cmd/server/` + `internal/{config,log,mcpserver,tools,transport}/` + buf configs.
- Committed placeholder `proto/gen/` files so a fresh clone compiles before `make generate`.
