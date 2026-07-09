//! `scafgen self-update` 命令 —— 从 GitHub Release 原地升级二进制。
//!
//! 使用 `self_update` crate 的 github 后端（rustls，无 openssl）。Release 资产由
//! release.yml 产出，命名为 `scafgen-<version>-<target>.{tar.gz|zip}`，资产名内含
//! target triple，self_update 据此自动匹配当前平台对应资产（6 个 target：
//! x86_64/aarch64 × linux-musl/apple-darwin/windows-msvc）。
//!
//! 进度/状态走 tracing(stderr)，与项目其余输出一致；实际网络更新只在本处理函数
//! 内发生（单元测试只覆盖参数解析，不触网）。

use anyhow::{Context, Result};

const REPO_OWNER: &str = "sunerpy";
const REPO_NAME: &str = "scaffold-gen";
const BIN_NAME: &str = "scafgen";

/// `scafgen self-update` 入口。
///
/// `self_update` 是同步阻塞 crate，内部自建 reqwest/tokio 运行时；若直接在
/// `#[tokio::main]` 的异步上下文里调用，其运行时 drop 时会 panic
/// （"Cannot drop a runtime in a context where blocking is not allowed"）。
/// 因此把全部阻塞工作放到一条独立 OS 线程执行，使其脱离外层异步上下文。
///
/// - `check`：仅查询是否有更新版本，不安装。
/// - `force`：跳过确认、即使已是最新也重装。
/// - `tag`：安装指定版本标签（如 `v0.2.0`），而非 latest。
pub fn execute(check: bool, force: bool, tag: Option<String>) -> Result<()> {
    std::thread::scope(|scope| {
        scope
            .spawn(|| run_blocking(check, force, tag))
            .join()
            .map_err(|_| anyhow::anyhow!("self-update worker thread panicked"))?
    })
}

fn strip_v(s: &str) -> &str {
    s.strip_prefix('v').unwrap_or(s)
}

fn is_same_version(current: &str, latest: &str) -> bool {
    strip_v(current) == strip_v(latest)
}

/// Convert a release `version` (as returned by `get_latest_release().version`,
/// which has any leading `v` stripped — see self_update github.rs
/// `from_release`: `tag.trim_start_matches('v')`) into the GitHub release tag
/// form that `target_version_tag` expects.
///
/// `target_version_tag` is matched against the GitHub release **tag** verbatim:
/// self_update's `get_release_version` builds `/repos/<o>/<r>/releases/tags/{ver}`
/// inserting the value as-is. This repo's release.yml tags releases as `v*`
/// (e.g. `v0.7.0`), so the bare version `0.7.0` must be prefixed with `v`.
/// If the version already carries a `v` (defensive — it should not), it is left
/// untouched to avoid producing `vv0.7.0`.
fn release_version_to_tag(version: &str) -> String {
    if version.starts_with('v') {
        version.to_owned()
    } else {
        format!("v{version}")
    }
}

/// Resolve a GitHub auth token from the two accepted environment sources,
/// following the gh CLI convention: `GITHUB_TOKEN` takes precedence, falling
/// back to `GH_TOKEN`. A value that is empty (or whitespace-only after trim) is
/// treated as unset. Returns `None` when neither yields a usable token.
///
/// Kept a **pure function** (takes the already-read `Option<&str>` values, never
/// touches the global process environment) so it is unit-testable without env
/// mutation. `build_updater` reads the real env vars and passes them in.
fn resolve_auth_token(github: Option<&str>, gh: Option<&str>) -> Option<String> {
    github
        .into_iter()
        .chain(gh)
        .map(str::trim)
        .find(|t| !t.is_empty())
        .map(str::to_owned)
}

fn build_updater(
    current: &str,
    force: bool,
    target_tag: Option<&str>,
) -> Result<Box<dyn self_update::update::ReleaseUpdate>> {
    let mut builder = self_update::backends::github::Update::configure();
    builder
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(BIN_NAME)
        .current_version(current)
        .show_download_progress(true)
        .no_confirm(force);
    if let Some(tag) = target_tag {
        builder.target_version_tag(tag);
    }
    // Feed an auth token to the GitHub backend so requests count against the
    // authenticated 5000/hr quota instead of the anonymous 60/hr limit (the
    // shared-egress-IP rate limit that surfaces as HTTP 403). `self_update`
    // 0.42 exposes `.auth_token()` but never reads env itself, so wire it here.
    // The token is only read from the environment — never logged, never
    // persisted — and does not affect `show_download_progress`.
    let github_token = std::env::var("GITHUB_TOKEN").ok();
    let gh_token = std::env::var("GH_TOKEN").ok();
    if let Some(token) = resolve_auth_token(github_token.as_deref(), gh_token.as_deref()) {
        builder.auth_token(&token);
    }
    builder
        .build()
        .context("configuring the self-update backend")
}

/// Query the latest GitHub release, attaching an **actionable** error context
/// on failure. The bare `get_latest_release()` failure is most often GitHub API
/// rate limiting (anonymous 60/hr), which `self_update` 0.42 surfaces without a
/// structured status code — so rather than brittly matching "403"/"rate limit"
/// strings we uniformly enrich the context with recovery guidance. Shared by
/// both the `--check` and the update (latest-resolution) paths so the two never
/// drift. Returns the same `Release` type as `get_latest_release()`, keeping
/// callers' `.version` field usage unchanged.
fn latest_release(
    updater: &dyn self_update::update::ReleaseUpdate,
) -> Result<self_update::update::Release> {
    updater.get_latest_release().context(
        "querying the latest GitHub release failed. \
         This is often GitHub API rate limiting (anonymous requests are capped at 60/hour). \
         查询最新 GitHub Release 失败，通常是 GitHub API 速率限制（匿名请求每小时仅 60 次）。\n\
         Fixes / 解决办法:\n\
         - Set GITHUB_TOKEN to raise the quota to 5000/hour, e.g. \
         `export GITHUB_TOKEN=$(gh auth token)` 设置 GITHUB_TOKEN 提高配额；\n\
         - retry later 稍后重试；\n\
         - or upgrade manually via `cargo install scaffold-gen --force` 或用 cargo 手动升级。",
    )
}

fn run_blocking(check: bool, force: bool, tag: Option<String>) -> Result<()> {
    use self_update::cargo_crate_version;

    let current = cargo_crate_version!();

    if check {
        // `--check` reports against the TRUE latest release: `get_latest_release`
        // hits `/releases/latest` (no SemVer-compat filter), so a newer MAJOR is
        // still reported here. `bump_is_greater` is only a message heuristic —
        // it must never block the actual update path below.
        let updater = build_updater(current, force, None)?;
        let latest = latest_release(updater.as_ref())?;
        if self_update::version::bump_is_greater(current, &latest.version).unwrap_or(false) {
            tracing::info!("{BIN_NAME} {current} -> {} available", latest.version);
            tracing::info!("run `{BIN_NAME} self-update` to install it");
        } else {
            tracing::info!("{BIN_NAME} {current} is up to date");
        }
        return Ok(());
    }

    // Resolve the effective target tag:
    //   - explicit `--tag` always wins (pin exactly what the user asked for);
    //   - otherwise pin the TRUE latest release so `update()` installs it
    //     DIRECTLY, bypassing self_update's SemVer-compatible filter (which
    //     would otherwise refuse a newer MAJOR, e.g. v0.x -> v1.0.0).
    let resolved_tag: String = match tag {
        Some(explicit) => explicit,
        None => {
            // A lightweight probe updater purely to query `/releases/latest`.
            let probe = build_updater(current, force, None)?;
            let latest = latest_release(probe.as_ref())?;
            // `latest.version` has the `v` stripped; restore the tag form.
            release_version_to_tag(&latest.version)
        }
    };

    // self_update 0.42's `update_extended` never checks current == target, so
    // when already on the latest release it still prints the download/replace
    // prompt. Short-circuit here; `--force` deliberately bypasses it to reinstall.
    // `resolved_tag` is the `v`-prefixed tag form; `is_same_version` normalizes
    // the leading `v` on either side so `v0.7.0` == current `0.7.0`.
    if !force && is_same_version(current, &resolved_tag) {
        tracing::info!("{BIN_NAME} {current} is already up to date");
        return Ok(());
    }

    let updater = build_updater(current, force, Some(&resolved_tag))?;
    let status = updater.update().context("performing the self-update")?;
    if status.updated() {
        tracing::info!("Updated {BIN_NAME} to {}", status.version());
    } else {
        tracing::info!("{BIN_NAME} {} is already up to date", status.version());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{is_same_version, release_version_to_tag, resolve_auth_token};

    // Network-free: these only assert the pure helpers that the `tag = None`
    // path relies on. The latest-resolution itself
    // (`get_latest_release` -> `release_version_to_tag` -> `target_version_tag`)
    // intentionally hits GitHub at runtime and is NOT exercised here.

    #[test]
    fn bare_version_gets_v_prefix() {
        // `get_latest_release().version` strips the leading `v` (self_update
        // github.rs `from_release`), so `0.7.0` must become the `v0.7.0` tag
        // that `/releases/tags/{tag}` is matched against.
        assert_eq!(release_version_to_tag("0.7.0"), "v0.7.0");
        assert_eq!(release_version_to_tag("1.0.0"), "v1.0.0");
        assert_eq!(release_version_to_tag("10.20.30"), "v10.20.30");
    }

    #[test]
    fn already_prefixed_version_is_left_intact() {
        // Defensive: never produce a doubled `vv` prefix.
        assert_eq!(release_version_to_tag("v0.7.0"), "v0.7.0");
    }

    #[test]
    fn prerelease_and_build_metadata_preserved() {
        assert_eq!(release_version_to_tag("1.2.0-rc.1"), "v1.2.0-rc.1");
    }

    #[test]
    fn same_bare_versions_are_equal() {
        assert!(is_same_version("0.7.0", "0.7.0"));
    }

    #[test]
    fn v_prefix_on_latest_is_ignored() {
        assert!(is_same_version("0.7.0", "v0.7.0"));
    }

    #[test]
    fn v_prefix_on_current_is_ignored() {
        assert!(is_same_version("v0.7.0", "0.7.0"));
    }

    #[test]
    fn different_minor_is_not_equal() {
        assert!(!is_same_version("0.7.0", "0.8.0"));
    }

    #[test]
    fn different_patch_is_not_equal() {
        assert!(!is_same_version("0.7.0", "0.7.1"));
    }

    #[test]
    fn auth_token_prefers_github_over_gh() {
        assert_eq!(
            resolve_auth_token(Some("x"), Some("y")),
            Some("x".to_owned())
        );
    }

    #[test]
    fn auth_token_falls_back_to_gh() {
        assert_eq!(resolve_auth_token(None, Some("y")), Some("y".to_owned()));
    }

    #[test]
    fn auth_token_uses_github_when_gh_absent() {
        assert_eq!(resolve_auth_token(Some("x"), None), Some("x".to_owned()));
    }

    #[test]
    fn auth_token_none_when_both_absent() {
        assert_eq!(resolve_auth_token(None, None), None);
    }

    #[test]
    fn auth_token_skips_empty_github_and_uses_gh() {
        assert_eq!(
            resolve_auth_token(Some(""), Some("y")),
            Some("y".to_owned())
        );
    }

    #[test]
    fn auth_token_whitespace_only_is_treated_as_unset() {
        assert_eq!(resolve_auth_token(Some("  "), None), None);
    }
}
