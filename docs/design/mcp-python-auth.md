# mcp-python 可选鉴权层设计 / Optional MCP Auth Design

> **范围 / Scope**: 仅 `mcp-python` 框架（fastmcp + official 两后端）。可选、配置驱动、默认关闭。
> **状态 / Status**: 设计文档（实现见 `.omo/plans/mcp-python-auth.md`）。
> **API 来源 / API facts**: 全部摘自 `.omo/drafts/mcp-python-auth.md` 的 "Findings"（已从已安装源码逐项核实），未臆造。

## 1. 背景 / Problem

用户希望在生成的 mcp-python 服务里**可选地**接入企业级鉴权——Active Directory（AD）与 SSO，
使只有持有合法 token 的调用方才能访问工具。要求：

- 默认行为不变：不开启鉴权时生成的项目与今天**逐字节一致**，零额外代码 / 配置 / 依赖。
- 一套配置同时覆盖 AD（经 AD FS / Entra ID）与通用 SSO（Okta / Keycloak / Auth0）。
- 两个后端（fastmcp 默认、official 官方 SDK）共用同一配置面与同一 `jwt` 模式。

The user wants OPTIONAL enterprise auth (AD + SSO) in generated mcp-python servers; `--auth none`
(default) renders zero auth code; one config surface covers AD-via-ADFS/Entra and generic SSO; both
backends share it.

## 2. MCP 鉴权模型 / Auth model

MCP 采用 **OAuth 2.1 bearer** token。MCP 服务在该模型里是 **Resource Server（资源服务器）**：

- 只**校验** token，**不**为用户登录、**不**签发 token、**不**充当 Authorization Server。
- token 由外部 IdP（身份提供方）签发；服务只负责验证其有效性。
- **官方 SDK（official）后端**在开启鉴权后自动暴露 `/.well-known/oauth-protected-resource`（RFC 9728），向客户端声明"我是受保护资源，请去对应 AS 取 token"。
- **fastmcp 后端**在开启鉴权后校验 token，但不发布该 discovery 端点（JWTVerifier 只校验，不挂载发现路由）；客户端需直接配置 IdP issuer。

> [!NOTE]
> 鉴权**关闭**时观察到的 `GET /.well-known/openid-configuration 404` 属正常现象：客户端在探测
> Authorization Server 元数据；服务未开启鉴权，自然没有该端点。
> The `404` on `/.well-known/openid-configuration` (auth off) is just a client probing for AS
> metadata — the server is a resource server, not an AS.

MCP = OAuth 2.1 bearer; the MCP server is a **Resource Server** (validates tokens, does NOT log
users in or issue tokens); an external IdP issues tokens. **Official SDK** exposes
`/.well-known/oauth-protected-resource` (RFC 9728) for discovery; **fastmcp** validates tokens but
does NOT publish the discovery endpoint (JWTVerifier mounts no routes by design) — clients must be
configured directly with the IdP issuer URL.

## 3. AD 与 SSO 殊途同归 / Convergence

本地（on-prem）AD **没有**原生 OAuth 能力，须由 **AD FS** 或 **Microsoft Entra ID** 在前面承载
OIDC/OAuth。AD FS、Entra ID 以及通用 OIDC IdP（Okta / Keycloak / Auth0）**形态一致**：

- 均暴露 `/.well-known/openid-configuration` 与 JWKS（公钥集合）。
- 均签发 **JWT 形态的 access token**。

因此**服务端校验动作完全相同**：用 JWKS 验签 + 校验 `iss`（签发方）+ `aud`（受众）+ `exp`（过期）

- `scopes`（权限范围）。一个 `jwt` 模式即可覆盖 AD 与 SSO，无需 AD / SSO 两套代码路径。

On-prem AD has no native OAuth → fronted by AD FS or Entra ID; both, plus generic OIDC IdPs, expose
`/.well-known/openid-configuration` + JWKS and issue JWT access tokens → server-side validation is
identical (JWKS signature + iss + aud + exp + scopes) → ONE `jwt` mode covers both.

具体 well-known 地址 / Concrete well-known URLs:

| IdP                     | `/.well-known/openid-configuration`                                                |
| ----------------------- | ---------------------------------------------------------------------------------- |
| AD FS                   | `https://<adfs-host>/adfs/.well-known/openid-configuration`                        |
| Entra ID                | `https://login.microsoftonline.com/<tenant>/v2.0/.well-known/openid-configuration` |
| Okta / Keycloak / Auth0 | 各 issuer 自身的 `<issuer>/.well-known/openid-configuration`                       |

从该文档读出 `jwks_uri` 与 `issuer`；`audience` 取 IdP 为本 API/资源签发 token 时写入的标识。

## 4. SAML 注意 / SAML caveat

纯 **SAML** 不能直接用于 MCP 的 OAuth 2.1 模型——SAML 是 XML 断言，不签发 OAuth bearer token。
若组织只有 SAML IdP，需要一个 **OIDC 桥接**（如 Entra ID / Keycloak 把 SAML 转成 OIDC）才能接入。

**本设计不实现 SAML**，仅记录该桥接注意事项。SAML 属 OUT OF SCOPE。

Pure SAML is NOT usable by MCP's OAuth 2.1 model; it needs an OIDC bridge (Entra ID / Keycloak).
SAML is documented-but-out — OUT OF SCOPE.

## 5. 两后端 API / Per-backend API

两个后端共用同一 `[auth]` 配置，仅"如何构造 verifier"不同。

### fastmcp 后端（默认 / default）

fastmcp 自带 JWKS verifier，无需额外依赖：

```python
from fastmcp.server.auth.providers.jwt import JWTVerifier

verifier = JWTVerifier(
    jwks_uri=settings.auth.jwks_uri,
    issuer=settings.auth.issuer,
    audience=settings.auth.audience,
    algorithm=settings.auth.algorithm,
    required_scopes=settings.auth.required_scopes,
)
mcp = FastMCP("<project>", auth=verifier)
```

> `JWTVerifier` 中 `public_key` 与 `jwks_uri` 互斥（XOR），二者只能取其一，否则报错。
> 设置 `auth=` 后，fastmcp 自动挂载 bearer 中间件并注册受保护资源元数据路由。

### official 官方 SDK 后端（pinned v1）

官方 v1 **没有**内置 JWKS verifier，因此自带一个用 `pyjwt[crypto]` 实现的 `JwksTokenVerifier`：

```python
from mcp.server.auth.settings import AuthSettings
from app.auth import JwksTokenVerifier  # 本项目随 auth 一起生成

verifier = JwksTokenVerifier(...)
auth_settings = AuthSettings(
    issuer_url=settings.auth.issuer,                 # AnyHttpUrl
    resource_server_url=settings.auth.resource_server_url,  # AnyHttpUrl，本服务公网基址
    required_scopes=settings.auth.required_scopes or None,
)
mcp = FastMCP(
    "<project>",
    json_response=True,
    stateless_http=settings.mcp.stateless,
    auth=auth_settings,
    token_verifier=verifier,
)
```

- `AuthSettings(issuer_url=, resource_server_url=, required_scopes=)`，其中 `issuer_url` 与
  `resource_server_url` 均为 `AnyHttpUrl`（必须是合法 http(s) URL）。设置 `auth` 时，
  必须且只能提供 `auth_server_provider` 或 `token_verifier` 之一——本项目提供 `token_verifier`。
- 自带 verifier 实现 `TokenVerifier` 协议：`async def verify_token(self, token: str) -> AccessToken | None`，
  内部用 `PyJWKClient` 取签名公钥、`jwt.decode(...)` 验签并映射 claims 到 `AccessToken`。

fastmcp uses its built-in `JWTVerifier(jwks_uri/issuer/audience/algorithm/required_scopes)` +
`FastMCP(auth=verifier)`. The official SDK v1 uses
`AuthSettings(issuer_url/resource_server_url/required_scopes)` + a custom
`TokenVerifier.verify_token() -> AccessToken`; v1 ships NO built-in JWKS verifier, so we ship one
(`JwksTokenVerifier`) built on `pyjwt[crypto]`.

### BLOCKER：拼接父应用须重挂鉴权中间件 / Splice must re-attach auth middleware

**已核实事实 / Verified fact**：两个后端都把 bearer 认证作为**应用级（app-level）中间件**挂在
**transport 子应用**的 Starlette 上——

- official：`AuthenticationMiddleware(BearerAuthBackend)` + `AuthContextMiddleware`（server.py:1041-1043）；
- fastmcp：`auth.get_middleware()` 经 `create_base_app(middleware=)` 挂载。

而本项目修 307 用的 `_build_routes()` 路由拼接只复制了子应用的 `.routes`、**丢掉了 `.middleware`**。
后果：拼接后的父应用里 `scope["user"]` 永不被设置，路由上的 `RequireAuthMiddleware`
（bearer_auth.py:103）会对**每个请求**（即便携带合法 token）一律返回 401。

**修复 / Fix**：开启鉴权时，把子应用（streamable / sse）的认证中间件一并重挂到父
`Starlette(..., middleware=...)` 上（按类型去重，因两个子应用共享同一 verifier，中间件一致）。
关闭鉴权时不重挂任何中间件，保持 `--auth none` 输出纯净。该修复的决定性验证是：携带合法 token
的请求返回 **200 且响应体含 echo 结果**（而非仅"非 401"）。

Both backends attach bearer auth as app-level `middleware=` on the transport sub-app; the 307-fix
route-splice copies only `.routes` and drops `.middleware`, so a valid token is wrongly 401'd. The
fix re-attaches the sub-apps' auth middleware onto the spliced parent.

## 6. 配置面 / Config surface

### `[auth]` 配置块（`config.toml`，默认全关）

| 键 / Key              | 类型 / Type | 默认 / Default | 说明 / Notes                                                                                                                              |
| --------------------- | ----------- | -------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `enabled`             | bool        | `false`        | 运行期总开关；`false` 时即便生成了 auth 代码也以无鉴权运行                                                                                |
| `mode`                | str         | `"jwt"`        | 目前仅 `jwt`                                                                                                                              |
| `jwks_uri`            | str         | `""`           | 取自 IdP `/.well-known/openid-configuration` 的 `jwks_uri`                                                                                |
| `issuer`              | str         | `""`           | 取自同上的 `issuer`；official 后端要求合法 http(s) URL                                                                                    |
| `audience`            | str         | `""`           | IdP 为本 API/资源写入 token 的受众标识（可为裸 URI/GUID）                                                                                 |
| `resource_server_url` | str         | `""`           | **本服务自身**的公网基址（如 `http://host:port`），喂给 `AuthSettings.resource_server_url` 与受保护资源元数据；**不可**与 `audience` 混用 |
| `required_scopes`     | list[str]   | `[]`           | 必需的权限范围；空表示不校验 scope                                                                                                        |
| `algorithm`           | str         | `"RS256"`      | 验签算法                                                                                                                                  |

> [!IMPORTANT]
> `resource_server_url` 是**本服务公网 URL**，不可由 `audience` 派生：`audience` 可能是裸标识
> （URI/GUID），而 `resource_server_url` 必须是合法 http(s) URL。

### CLI 与运行期 / CLI and runtime

- 生成期标志 `--auth <none|jwt>`，**默认 `none`**；仅 mcp-python 框架提供该标志与交互式 prompt。
- `none`：渲染**零**鉴权代码 / 配置 / 依赖（与今日输出一致）；`jwt`：渲染 auth 配置 + 两后端接线 +（official）`app/auth.py` + pyjwt 依赖；运行期仍受 `[auth] enabled` 二次控制。
- 运行期 `enabled` 开关：生成的 auth-capable 项目可经 `config.toml` / `.env` 把鉴权关掉。

### Fail-fast 与缺省语义 / Fail-fast and absent-table semantics

- **Fail-fast**：当 `enabled = true` 但 `jwks_uri` / `issuer` / `resource_server_url` 任一为空，
  启动时抛清晰错误，**绝不**静默以无鉴权运行。
- **缺省 `[auth]` 表 = 关闭，不是错误**：`--auth jwt` 生成的项目若 `config.toml` 中**没有**
  `[auth]` 表，`AuthConfig()` 默认令 `enabled=false`，服务以**无鉴权**运行——这是预期行为
  （absent table = disabled），fail-fast 仅在 `enabled=true` 但字段不全时触发。

### 环境变量 / Env overrides

- 嵌套分隔符 `AUTH__`，如 `AUTH__ENABLED`、`AUTH__JWKS_URI`、`AUTH__ISSUER`、
  `AUTH__AUDIENCE`、`AUTH__RESOURCE_SERVER_URL`。
- **列表型值经环境变量必须是 JSON**：`AUTH__REQUIRED_SCOPES=["mcp.read","mcp.call"]`
  （pydantic-settings 把 `__` 嵌套的 list 字段按 JSON 解析）。

The `[auth]` block keys + the `--auth <none|jwt>` flag (default none) + a runtime `enabled` toggle;
fail-fast when enabled but `jwks_uri`/`issuer`/`resource_server_url` missing; an absent `[auth]`
table means disabled (not an error); env list values (`AUTH__REQUIRED_SCOPES`) must be JSON.

## 7. 决策记录 / Decisions

用户锁定的 4 项决策 / The 4 user-locked decisions:

1. **统一 `jwt` 模式**：仅 JWKS 资源服务器校验；**不**纳入 fastmcp 的
   `AzureProvider` / `entra` 等托管 OAuth-proxy provider。
   Unified `jwt`/JWKS mode only — no AzureProvider/entra.
2. **official 后端自带 pyjwt verifier**：官方 v1 无内置 JWKS verifier，故写一个
   `pyjwt[crypto]` 实现的 `JwksTokenVerifier`（~30 行）。
   The official backend gets a small pyjwt-based verifier.
3. **`--auth` 默认 `none`**：默认渲染零鉴权代码，现有项目与行为不受影响。
   `--auth` defaults to none.
4. **SAML 记录但不实现**：纯 SAML 需 OIDC 桥接，OUT OF SCOPE。
   SAML documented-but-out.

附加 / Plus:

- **pyjwt 仅 official 后端**：`pyjwt[crypto]` 只在 `--auth jwt` **且** official 后端时加入依赖；
  fastmcp 的内置 `JWTVerifier` 自带 crypto（authlib/cryptography），故 fastmcp+auth **不**引入
  pyjwt。pyjwt is official-backend-only.

## 8. 来源 / References

已核实来源 / Verified sources:

- **MCP 授权规范**：modelcontextprotocol.io — Authorization（OAuth 2.1 resource server 模型、
  `/.well-known/oauth-protected-resource` / RFC 9728）。
- **fastmcp**：gofastmcp.com/servers/auth；安装源 `fastmcp/server/auth/providers/jwt.py:184`
  （`JWTVerifier` 签名）。
- **official Python SDK**：github.com/modelcontextprotocol/python-sdk（`mcp/server/auth`）；
  安装源：
  - `mcp/server/auth/settings.py:15`（`AuthSettings`；`issuer_url` / `resource_server_url` 为 `AnyHttpUrl`）
  - `mcp/server/auth/provider.py:39`（`AccessToken` 字段）`,96`（`TokenVerifier.verify_token`）
  - `mcp/server/fastmcp/server.py:218-224`（auth XOR token_verifier）`,1041-1043`
    （app 级认证中间件 + 路由上的 `RequireAuthMiddleware`）
  - `mcp/server/auth/middleware/bearer_auth.py:103`（读取 `scope["user"]`）
- **AD FS OIDC 概念**：learn.microsoft.com（AD FS OpenID Connect / OAuth concepts；
  `/authorize` `/token` `/keys` `/.well-known/openid-configuration` 端点形态）。
