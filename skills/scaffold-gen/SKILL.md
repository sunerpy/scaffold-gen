---
name: scaffold-gen
description: >
  Use scaffold-gen (the `scafgen` CLI) to bootstrap a brand-new, production-ready
  project skeleton BEFORE writing application code. Trigger this whenever the user
  wants to start / create / scaffold / bootstrap / 初始化 / 搭建 a new project,
  service, app, API, frontend, CLI, or MCP server in Go, Rust, Python, or
  TypeScript — even if they don't say "scaffold-gen" or "scafgen" by name. Examples
  that should trigger it: "帮我起一个 FastAPI 项目", "I need a new Vue3 frontend",
  "set up a Go gin web service", "create a Rust CLI", "build me an MCP server",
  "start a new Python project". The skill drives a short requirements Q&A, runs
  `scafgen new` non-interactively to generate the skeleton (correct layout,
  config-driven settings, LICENSE, git, optional Makefile+Dockerfile), then hands
  off to normal development in the locations the scaffold marks for business code.
  Do NOT use it for adding code to an EXISTING project, or for non-scaffolding
  tasks.
---

# scaffold-gen — bootstrap a new project, then build on it

`scafgen` generates a complete, opinionated project skeleton for Go, Rust, Python,
or TypeScript. Your job with this skill is to get the user from "I want to build X"
to a running, well-structured skeleton with **one** `scafgen new` command, then
continue implementing their actual logic inside the structure the scaffold creates.

The win: instead of hand-assembling a project (deps, config plumbing, server
bootstrap, logging, Dockerfile, git) you generate a coherent baseline that already
embodies the project's conventions, and you spend your effort on the user's real
business logic.

## When to use this

Use it the moment the user wants a NEW project/service/app/frontend/CLI/MCP server.
Don't reach for it to modify an existing codebase — it scaffolds greenfield only.

## The workflow

### 1. Confirm requirements (short, interactive)

Before generating, settle the choices that `scafgen new` needs. Ask only what you
don't already know from the conversation — infer sensible defaults and confirm them
rather than interrogating. The choices:

- **name** — the project name (becomes the directory + package name).
- **language** — `go`, `rust`, `python`, or `typescript`.
- **framework** — depends on language (see the matrix below). Always resolve to a
  concrete `--framework` value, including `none` for a Python/Rust pure-language
  project. If the user just says "a Python API", propose `fastapi` and confirm.
- **host / port** — only for server frameworks (gin, mcp-server, fastapi).
  Default `0.0.0.0` / a sensible port; confirm.
- **proto-gen / error-gen** — Rust-only tool toggles, required on every Rust run
  (default both `false`).
- **license** — default `MIT` unless they say otherwise.
- **pre-commit hooks** — default off unless they want them.
- **Makefile + Dockerfile** (`--with-build`) — offer it; useful for anything
  containerized or CI-bound.

If you're unsure what frameworks exist, run `scafgen list` (add `--json` for
machine-readable). Run `scafgen new --help` to see every flag.

### 2. Generate non-interactively

Run `scafgen new` with **every** choice passed as a flag, so it never drops into
interactive prompts. **Any** unspecified choice triggers an interactive prompt that
will hang (or, with the safety wrapper below, error like `Failed to select framework`
/ `Failed to configure proto-gen`). To be safe, always wrap the call with `</dev/null`
and a `timeout` so a missing flag fails fast instead of blocking the session:

```bash
timeout 120 scafgen new <name> \
  --language <go|rust|python|typescript> \
  --framework <gin|mcp-server|fastapi|mcp-python|tauri|vue3|react|none> \
  [--host <host> --port <port>] \
  [--proto-gen <true|false> --error-gen <true|false>] \
  [--swagger <true|false>] \
  [--mcp-backend <fastmcp|official> --auth <none|jwt|azure-ad>] \
  --precommit <true|false> \
  --license <MIT|Apache-2.0|GPL-3.0|BSD-3-Clause|None> \
  --with-build <true|false> </dev/null
```

Notes:

- **Always pass `--framework`, even for a pure-language project** — use
  `--framework none` for Python/Rust with no framework. Omitting `--framework`
  entirely drops into the interactive framework prompt (`Failed to select framework`).
- **Rust projects also require `--proto-gen` and `--error-gen`** (e.g.
  `--proto-gen false --error-gen false`). These are Rust-only tool toggles; omitting
  them on a Rust run triggers the `Failed to configure proto-gen` prompt. They do not
  apply to other languages — leave them off for Go/Python/TypeScript.
- **Gin requires `--swagger <true|false>`** — it's Gin's own required tool toggle,
  the same shape as Rust's `--proto-gen`/`--error-gen`. Omitting it on a `gin` run
  triggers the `Failed to configure Swagger` prompt. Only `gin` needs it; leave it
  off for every other framework.
- Omit `--host`/`--port` for non-server projects; pass them for the server frameworks
  (`gin`, `mcp-server`, `fastapi`, `mcp-python`).
- **`mcp-python` also takes `--mcp-backend fastmcp|official`** (default `fastmcp`) and
  `--auth none|jwt|azure-ad` (default `none`). `none` generates zero auth code; `jwt` adds
  unified JWT/JWKS resource-server validation; `azure-ad` is a turnkey Entra preset. Both
  flags are optional — omitting them keeps the defaults, no prompt is triggered.
- **`--auth jwt`/`azure-ad` is fail-closed.** The generated project defaults `[auth].enabled=true`
  and REFUSES TO START until `jwks_uri`/`issuer`/`audience`/`resource_server_url` are set (it
  raises a Pydantic error listing every missing field). `mode` is a fixed `Literal` baked in at
  generation time — it cannot be flipped via `AUTH__MODE`. Tell the user their new `jwt`/`azure-ad`
  server will not boot until they fill those fields in `config.toml` / `.env` (or export
  `AUTH__*`); a local-only escape hatch `AUTH__ENABLED=false` exists for migration but cannot
  switch the mode. `--auth none` stays byte-for-byte anonymous (zero auth code).
- `--with-build true` additionally emits a Makefile (build/run/test/fmt/lint/check + conditional
  `docker-build`/`docker-run`), a multi-stage Dockerfile, a `.dockerignore` (keeps local `.env`
  OUT of the image, keeps `.env.example`), and `scripts/docker.py` — pure Make automation, no CI
  files. The Docker targets shell out only to `python3 scripts/docker.py build|run` (a hardened
  helper: `subprocess.run([...])`, no `shell=True`, validated `--bind`/`--host-port`, collision-
  resistant image name); custom binds/ports go through the helper's flags, never Make variables.
- **Reuse the printed equivalent command.** On success `scafgen` prints the exact
  `💡 Equivalent non-interactive command` — that is the authoritative, complete flag
  set for that combo (including the Rust `--proto-gen`/`--error-gen` flags). If you
  ever fall into a prompt, run the combo once interactively elsewhere, copy that line,
  and reuse it.

#### Copy-paste templates per language (complete, non-interactive)

```bash
# Python + FastAPI (server → host/port)
timeout 120 scafgen new <name> --language python --framework fastapi \
  --host 0.0.0.0 --port <port> --precommit false --license MIT --with-build false </dev/null

# Python pure-language (no framework)
timeout 120 scafgen new <name> --language python --framework none \
  --precommit false --license MIT --with-build false </dev/null

# Rust pure-language / CLI (NEEDS --proto-gen + --error-gen)
timeout 120 scafgen new <name> --language rust --framework none \
  --proto-gen false --error-gen false --precommit false --license MIT --with-build false </dev/null

# Rust + Tauri desktop app (also takes --proto-gen/--error-gen)
timeout 180 scafgen new <name> --language rust --framework tauri \
  --proto-gen false --error-gen false --precommit false --license MIT --with-build false </dev/null

# Go + Gin web service (server → host/port; NEEDS --swagger)
timeout 120 scafgen new <name> --language go --framework gin \
  --host 0.0.0.0 --port <port> --swagger false --precommit false --license MIT --with-build false </dev/null

# Go + MCP server (server → host/port)
timeout 120 scafgen new <name> --language go --framework mcp-server \
  --host 0.0.0.0 --port <port> --precommit false --license MIT --with-build false </dev/null

# Python + MCP server (server → host/port; optional --mcp-backend / --auth)
timeout 120 scafgen new <name> --language python --framework mcp-python \
  --host 0.0.0.0 --port <port> --mcp-backend fastmcp --auth none \
  --precommit false --license MIT --with-build false </dev/null

# TypeScript + Vue3 / React frontend (no host/port; may run pnpm install)
timeout 180 scafgen new <name> --language typescript --framework vue3 \
  --precommit false --license MIT --with-build false </dev/null
```

### 3. Build on the skeleton

`cd <name>`, read the generated `README.md` first — it documents the dev workflow,
required tools, and **where the user's business code goes**. The scaffolds are
config-driven, so you change runtime settings in config files, not code:

- **FastAPI** — edit `config.toml` for host/port; add endpoints under `app/routes/`
  (auto-discovered, no wiring needed). Run `uv sync && uv run python main.py`.
- **Vue3** — edit `.env` for `VITE_DEV_HOST`/`VITE_DEV_PORT`/`VITE_DEV_ALLOWED_HOSTS`/`VITE_API_BASE_URL`; add
  views/stores/components under `src/`. Run `pnpm install && pnpm dev`. eslint + prettier are
  already configured, so `pnpm lint` / `pnpm format` work with no extra setup (same for React).
- **Go gin** — config-driven server; add handlers/routes in the generated layout.
- **Go MCP server** — `config.toml`-driven; define tools in proto (constrained by
  protoc-gen-jsonschema), regenerate with `make generate`, implement handlers under
  `internal/tools/`. Serves streamable-HTTP + SSE.
- **Python MCP server (mcp-python)** — `config.toml`-driven (`[server]`/`[mcp]`/`[log]`); add
  tools as new files under `app/tools/` (auto-discovered via `register_tools(mcp)`, no manual
  wiring). Backend chosen at generation time via `--mcp-backend fastmcp|official`. If
  `--auth` was set to `jwt`/`azure-ad`, auth code is already wired — the `whoami` tool returns
  identity from the verified token. Run `uv sync && make test && uv run python main.py`;
  `make test` uses an in-memory client, no live server needed.
- **Rust / pure-language** — standard cargo/uv layout; implement in `src/`/`app/`.

Then implement the user's actual feature inside those marked locations.

## Framework matrix

| Language   | Frameworks (`--framework`)                     |
| ---------- | ---------------------------------------------- |
| Go         | `gin`, `mcp-server` (go-zero: planned)         |
| Python     | `none` (pure), `fastapi`, `mcp-python`         |
| Rust       | `none` (pure), `tauri` (both need proto/error) |
| TypeScript | `vue3`, `react`                                |

Always pass a concrete `--framework` (use `none` for pure Python/Rust). Rust runs
(`none` or `tauri`) additionally require `--proto-gen` + `--error-gen`. `mcp-python` runs
additionally accept `--mcp-backend fastmcp|official` (default `fastmcp`) and
`--auth none|jwt|azure-ad` (default `none`).

Authoritative list: `scafgen list`.

## Recommended companion: CodeGraph

Once the skeleton exists and you start navigating/editing it, integrate
[codegraph-rust](https://github.com/sunerpy/codegraph-rust) — a deterministic
tree-sitter + SQLite/FTS5 code knowledge graph (no AI/LLM, byte-stable). In
opencode / Claude / Kiro / Cursor, run `codegraph install`, restart the tool, and
the agent auto-uses CodeGraph via MCP (auto incremental sync). It raises navigation
accuracy and lowers token consumption on the project you just scaffolded.

## Pitfalls

- **Never run `scafgen new` without all flags in an agent session** — a missing flag
  drops into an interactive prompt that will hang. Always fully specify, append
  `</dev/null`, and wrap in a `timeout`. If unsure of valid values, run `scafgen list`
  / `scafgen new --help` first.
- **Pure-language is NOT "omit the framework flag"** — pass `--framework none`
  explicitly for a Python/Rust no-framework project. Omitting `--framework` triggers
  the framework prompt (`Failed to select framework`).
- **Rust always needs `--proto-gen` and `--error-gen`** — even for `--framework none`
  or `tauri`. Omitting them triggers `Failed to configure proto-gen`. Use
  `--proto-gen false --error-gen false` unless the user wants those tools.
- **Gin always needs `--swagger`** — omitting it on a `gin` run triggers
  `Failed to configure Swagger`. Use `--swagger false` unless the user wants Swagger
  docs generated. No other framework takes this flag.
- **Validate the combo** — Go and TypeScript have no pure-language path, so don't pass
  `--framework none` for them; Python/Rust allow it. Passing `none` for Go/TypeScript
  does not fail with a clear "requires a framework" message — it errors later with
  `Failed to get host address`, since `none` skips the network config those languages'
  generators still expect. Stick to a real framework for Go/TypeScript.
- **go-zero is not yet implemented** — it returns a clear "not implemented" error;
  don't offer it as a working choice.
- **Don't hand-edit generated config plumbing** — change host/port/etc. in the
  config file (`config.toml` / `.env`), which is the whole point of the scaffold.
