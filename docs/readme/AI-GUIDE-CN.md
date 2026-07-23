# 结合 AI Agent 使用 scafgen

[English README](../../README.md) · [简体中文 README](README-CN.md)

## 1. 简介

`scafgen` 是一个用 Rust 编写的脚手架生成器，二进制名叫 `scafgen`，crate 名叫
`scaffold-gen`，当前版本 0.10.10。它覆盖 Go、Rust、Python、TypeScript 四种语言，共 9
个可用框架（外加一个未实现的 go-zero），所有模板都在编译期内嵌进二进制，运行时不依赖
任何外部文件。

对配合 AI agent（opencode / Claude / Cursor / Kiro）使用来说，`scafgen` 真正的价值在
于：装一次内嵌的 skill，之后你只要跟 agent 说一句"帮我起一个 FastAPI 项目"，它就会
自己完成一轮简短的需求确认，然后非交互式地跑一次 `scafgen new`，把项目骨架生成好，
你不用再记任何 flag。这份指南教你把这条链路搭起来：安装 → 升级 → 装 skill → 让 agent
帮你建项目。

## 2. 安装

**Linux / macOS**（一行脚本）：

```sh
curl -fsSL https://raw.githubusercontent.com/sunerpy/scaffold-gen/main/scripts/install.sh | sh
```

**通过 cargo**（任意装了 Rust 的平台）：

```sh
cargo install scaffold-gen
```

装完用 `scafgen version` 验证，正常会往 stdout 打印：

```
scaffold-gen 0.10.10
repository: https://github.com/sunerpy/scaffold-gen
```

## 3. 升级

`scafgen self-update` 会检查 GitHub 最新 release 并原地更新二进制：

```sh
scafgen self-update            # 检查并更新到最新版本
scafgen self-update --check    # 只检查，不安装
scafgen self-update --force    # 强制重装当前同一版本
scafgen self-update --tag v0.2.0   # 固定安装某个 tag
```

该命令走 GitHub API，受匿名限流影响（每个 IP 每小时 60 次请求）。设置
`GITHUB_TOKEN`（未设置时回退读取 `GH_TOKEN`）可以把限额提到 5000 次/小时，例如：

```sh
export GITHUB_TOKEN=$(gh auth token)
scafgen self-update
```

这个 token 只会从环境变量读取，绝不会写盘或记进日志。如果 `self-update` 本身因为
环境限制跑不起来，退回用 `cargo install scaffold-gen --force` 重装最新版即可。

## 4. 安装 / 升级 Skill（结合 AI 的关键一步）

这是让 agent "会用" scafgen 的核心动作。`scafgen skill install` 会把内嵌的
SKILL.md 写进检测到的 agent skill 目录（opencode/claude/cursor/kiro），装完重启一下
agent，它就会在你说"起一个新项目"之类的话时自动套用这份 skill。

```sh
scafgen skill install              # 安装到所有检测到的 agent（默认全局）
scafgen skill status               # 查看每个 agent 的安装状态
scafgen skill update --force       # 刷新为当前二进制内嵌的最新版本
scafgen skill uninstall            # 卸载
```

常用 flag：

- `--target <agent>`（可重复）：只装到指定 agent，取值 `opencode`/`claude`/`cursor`/`kiro`，默认装到所有检测到的 agent。
- `--global`（默认）/ `--local`：装到用户全局目录还是当前项目目录。
- `-y/--yes`：跳过 install/uninstall 的确认提示。
- `--force`：`update` 时强制覆盖本地修改。

skill 目录里会留一个 sidecar 标记文件 `.scaffold-gen-skill.json`，里面记着当前已安装
内容的 git-blob 哈希。下次二进制升级后内嵌的 SKILL.md 变了，`scafgen skill status`
会通过对比这个哈希判断"需要更新"，`scafgen skill update` 才知道该不该覆盖。

## 5. 用 AI 创建项目

装好 skill 之后，正常流程是这样的：你说想要什么项目，agent 先做一轮简短的需求确认
（语言、框架、host/port、license 等），然后**用 flag 把每一个选择都传给
`scafgen new`**，非交互式跑完。这一点很重要：只要漏传一个 flag，`scafgen` 就会掉进
交互式提示，在 agent 会话里会直接卡死。所以 agent 侧的每条命令都会带上 `</dev/null`
和 `timeout` 包裹，让漏 flag 的情况快速报错而不是卡住。

### 框架矩阵

```text
Go:
  Gin          Gin (Web Framework) [available]
  go-zero      go-zero (Microservice Framework) [not implemented]
  mcp-server   MCP Server (Gin + go-sdk, streamable-HTTP + SSE) [available]
Python:
  None         None (Pure Language Project) [available]
  fastapi      FastAPI (Config-driven API Framework) [available]
  mcp-python   MCP Server — Python (FastMCP, streamable-HTTP + SSE) [available]
Rust:
  None         None (Pure Language Project) [available]
  Tauri        Tauri (Desktop App Framework) [available]
TypeScript:
  Vue3         Vue3 (Frontend Framework) [available]
  React        React (Frontend Framework) [available]
```

（等价于自己跑一次 `scafgen list` 看到的结果。）

### flag 规则

- **永远显式传 `--framework`**，包括纯语言项目——Python/Rust 传 `--framework none`。
  漏传会掉进交互式框架选择提示。
- **Go 和 TypeScript 没有纯语言路径**，不要给它们传 `--framework none`。
- **Rust 项目永远要带 `--proto-gen` 和 `--error-gen`**（`none` 或 `tauri` 都一样），
  不管用户要不要这两个工具，先给 `false`。
- **Gin 永远要带 `--swagger`**，同理，不要就给 `false`。
- **服务器类框架**（`gin`/`mcp-server`/`fastapi`/`mcp-python`）需要传
  `--host`/`--port`；纯语言、Tauri、Vue3/React 不用。

### 复制即用的非交互式模板

```bash
# Python + FastAPI (server → host/port)
timeout 120 scafgen new <name> --language python --framework fastapi \
  --host 0.0.0.0 --port <port> --precommit false --license MIT --with-build false </dev/null

# Python pure-language
timeout 120 scafgen new <name> --language python --framework none \
  --precommit false --license MIT --with-build false </dev/null

# Rust pure-language / CLI (NEEDS --proto-gen + --error-gen)
timeout 180 scafgen new <name> --language rust --framework none \
  --proto-gen false --error-gen false --precommit false --license MIT --with-build false </dev/null

# Rust + Tauri
timeout 180 scafgen new <name> --language rust --framework tauri \
  --proto-gen false --error-gen false --precommit false --license MIT --with-build false </dev/null

# Go + Gin (server → host/port; NEEDS --swagger)
timeout 120 scafgen new <name> --language go --framework gin \
  --host 0.0.0.0 --port <port> --swagger false --precommit false --license MIT --with-build false </dev/null

# Go + MCP server (server → host/port)
timeout 120 scafgen new <name> --language go --framework mcp-server \
  --host 0.0.0.0 --port <port> --precommit false --license MIT --with-build false </dev/null

# Python + MCP server (server → host/port; optional --mcp-backend / --auth)
timeout 120 scafgen new <name> --language python --framework mcp-python \
  --host 0.0.0.0 --port <port> --mcp-backend fastmcp --auth none \
  --precommit false --license MIT --with-build false </dev/null

# TypeScript + Vue3 / React
timeout 180 scafgen new <name> --language typescript --framework vue3 \
  --precommit false --license MIT --with-build false </dev/null
```

`scafgen new` 生成成功后会打印一行 `💡 Equivalent non-interactive command`，那就是
这次交互选择对应的完整、权威的非交互命令。如果哪天 agent 掉进了提示、或者不确定某个
组合该带哪些 flag，跑一次交互模式，把这行复制下来复用即可。

## 6. mcp-python 认证是 fail-closed（重点，务必写清）

给 `mcp-python` 传 `--auth jwt` 或 `--auth azure-ad` 时，生成出来的项目会默认把
`[auth].enabled` 设成 `true`，并且**在鉴权配置补全之前拒绝启动**——`jwks_uri`、
`issuer`、`audience`、`resource_server_url` 这四项只要缺一个，启动时就会抛出一个
列出全部缺失字段的 Pydantic 校验错误，报错原文是：

```
Value error, [auth] enabled but required fields are empty: jwks_uri, issuer, audience, resource_server_url
```

`mode` 是生成时就写死的固定 `Literal["jwt"]` 或 `Literal["azure-ad"]`，**没法**再通过
`AUTH__MODE` 环境变量切换。要让服务跑起来，需要在 `config.toml` / `.env` 里填好这四项，
或者直接导出对应的 `AUTH__*` 环境变量，例如：

```sh
export AUTH__JWKS_URI="https://login.microsoftonline.com/<tenant>/discovery/v2.0/keys"
export AUTH__ISSUER="https://login.microsoftonline.com/<tenant>/v2.0"
export AUTH__AUDIENCE="api://<client-id>"
export AUTH__RESOURCE_SERVER_URL="https://your-mcp-server.example.com"
```

`azure-ad` 模式可以从 `AUTH__TENANT_ID`/`AUTH__RESOURCE_APP_ID` 推导出
jwks/issuer/audience，但 `AUTH__RESOURCE_SERVER_URL` 仍然是必填的。本地调试有一个
仅本地用的退路 `AUTH__ENABLED=false`，可以先关掉鉴权跑起来，但它**不能**改变已经
写死的 `mode`。

`--auth none` 是零鉴权代码——生成的文件里完全没有认证相关逻辑，跟不传 `--auth` 时
（默认值就是 `none`）逐字节一致。

## 7. --with-build 与容器化

`--with-build true` 会额外生成：

- 一个 Makefile：`build`/`run`/`test`/`fmt`/`lint`/`check`，外加按需出现的
  `docker-build`/`docker-run` 目标；
- 一个多阶段构建的 `Dockerfile`；
- 一个 `.dockerignore`：把本地的 `.env` 排除在镜像之外，但保留 `.env.example`；
- 一个 `scripts/docker.py`：加固过的辅助脚本，内部用
  `subprocess.run([...])`（不用 `shell=True`），对 `--bind`/`--host-port` 做了校验，
  生成的镜像名带上摘要以避免冲突（形如 `<slug>-<digest>:latest`）。

`--with-build false`（默认）不会生成上面任何一个文件；如果这次生成本身自带
Makefile（比如 FastAPI/mcp-python/Go MCP），那个 Makefile 也不会有 docker 相关目标。

如果要自定义绑定地址或端口，走辅助脚本自己的 flag，**不要**改 Makefile 变量：

```sh
python3 scripts/docker.py build
python3 scripts/docker.py run --bind 127.0.0.1 --host-port <port>
```

## 8. 生成后在哪写业务代码

- **FastAPI** —— 在 `app/routes/` 下新增文件即可（自动发现，无需手动注册路由），
  host/port 在 `config.toml` 里改；运行 `uv sync && uv run python main.py`。
- **Python MCP server（mcp-python）** —— 在 `app/tools/` 下新增工具文件，
  `register_tools` 自动发现，无需手动接线；配置全在 `config.toml`；
  `make test` 用内存客户端跑测试，不需要真的起一个服务进程。
- **Go Gin** —— 配置在 `config/*.toml`（`dev`/`prod`/`test`/`example` 各一份），
  路由和 handler 放在 `routers/` 里。
- **Go MCP server** —— 工具定义在 proto 里，跑 `make generate` 生成代码，实现放在
  `internal/tools/`。
- **Rust / 纯 Python** —— 标准的 cargo/uv 项目布局，直接在 `src/`（Rust）或
  `app/`（Python）里写业务逻辑。

## 9. 搭配 CodeGraph（可选）

脚手架生成之后，如果你要继续在这个新项目里让 agent 帮你导航、改代码，建议顺手装一下
[codegraph-rust](https://github.com/sunerpy/codegraph-rust)——一个基于
tree-sitter + SQLite 的确定性代码知识图谱，不依赖 LLM。跑 `codegraph install` 装好后
重启 agent，它会通过 MCP 自动用上这个图谱做增量同步，让 agent 在新项目里的导航更准、
消耗的 token 更少。
