# Design: Azure AD / Entra ID auth mode for mcp-python (`--auth azure-ad`)

**Status:** proposed (awaiting approval) **Date:** 2026-06-26 **Baseline:** `726966a`

## 1. Goal & scope

Add a third `AuthMode` — `azure-ad` — to the `mcp-python` framework, parallel to the
existing `none` / `jwt`. It is a **preset layer over the existing JWT/JWKS machinery**:
Entra ID is an OIDC IdP that issues RS256 JWTs with a JWKS endpoint, so the existing
`JwksTokenVerifier` (official) / `JWTVerifier` (fastmcp) already verify Entra tokens. The
new mode makes the Entra/Azure AD case **turnkey** by baking in the Entra-specific presets
proven in production by `AI/m365-oa`.

In scope:

- `AuthMode::AzureAd` enum variant + CLI `--auth azure-ad` + interactive prompt.
- Entra-derived config: `tenant_id` + `resource_app_id` → auto-build jwks_uri / issuers /
  audience; **dual issuer acceptance** (v2 `login.microsoftonline.com/{t}/v2.0` + v1
  `sts.windows.net/{t}/`).
- Identity extraction from verified token: `preferred_username | upn | email | unique_name`,
  plus `oid` as stable id.
- **JWKS warm-up** at startup (cross-border Entra latency optimisation).
- **Security core**: trust ONLY the verified-token identity; the example tool demonstrates
  reading the authed identity, never an AI-supplied `user_email` param.
- Both backends: official + fastmcp.
- Docs note for the deferred **OAuth proxy** (Amazon Quick `resource`-rewrite hack) — NOT
  implemented; documented in `docs/` + `tmp/` for a future contributor.

Out of scope (this iteration):

- OAuth proxy / Amazon Quick `resource` rewrite (documented only).
- Any non-Python framework (no Node/TS MCP server).
- Opaque-token / introspection IdPs (e.g. NWCD SSO) — separate future mode.

## 2. Entra facts (verified in m365-oa prod)

| Item      | Value                                                                        |
| --------- | ---------------------------------------------------------------------------- |
| JWKS      | `https://login.microsoftonline.com/{tenant}/discovery/v2.0/keys`             |
| issuer v2 | `https://login.microsoftonline.com/{tenant}/v2.0`                            |
| issuer v1 | `https://sts.windows.net/{tenant}/`                                          |
| audience  | `{resource_app_id}` OR `api://{resource_app_id}`                             |
| identity  | `preferred_username` \| `upn` \| `email` \| `unique_name`; `oid` = stable id |
| algorithm | RS256                                                                        |

## 3. Key design decisions

1. **`azure-ad` is a superset preset of `jwt`, not a fork.** It reuses the same verifier
   classes. The difference is config derivation (tenant→endpoints) + dual-issuer + identity
   extraction + warm-up. Where the verifier needs to accept _two_ issuers, we extend the
   existing verifier to accept a list of issuers (jwt mode passes one; azure-ad passes two).
2. **`auth_enabled` stays the gate.** `is_enabled()` returns true for both `Jwt` and
   `AzureAd`. A new `auth_mode` context value `"azure-ad"` + a derived
   `auth_is_azure_ad` boolean drives Entra-specific template branches.
3. **`--auth none` byte-identical guarantee preserved.** No azure-ad code renders unless
   selected. The existing `mcp_python_auth_renders` test's none-path assertions must still hold.
4. **Config surface:** azure-ad adds `tenant_id` + `resource_app_id` to `[auth]`; jwks_uri /
   issuer(s) / audience are auto-derived in settings if azure-ad and left blank, but remain
   overridable. `resource_server_url` still required (official RFC 9728 endpoint).
5. **Security pattern is the headline feature** — generated example tool shows
   `get_authed_identity()` usage and a comment: never trust AI-supplied user params.

## 4. Open items for approval

- Identity helper exposure: a small `app/auth.py` helper (`current_identity()` reading the
  verified token / request context) for the official backend; fastmcp exposes its own
  `get_access_token()`. Confirm we add a backend-agnostic thin wrapper.
- Whether the example tool is modified (shows identity) or a NEW example tool is added.
  Proposal: add a NEW `whoami` example tool gated on azure-ad, leaving existing echo tool intact.
