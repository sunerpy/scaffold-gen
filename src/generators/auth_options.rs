//! mcp-python 鉴权模式选择 —— `AuthMode` 枚举。
//!
//! 决定生成的 mcp-python 项目是否内置可选的 JWT/JWKS 资源服务器鉴权：
//! - `None`：不启用（默认），渲染零鉴权代码 —— 与历史输出字节一致。
//! - `Jwt`：统一 JWT/JWKS 校验路径，覆盖 AD FS / Entra ID / Okta / 通用 OIDC。
//! - `AzureAd`：Entra ID / Azure AD 开箱即用预设，复用同一 JWT/JWKS 校验（双 issuer v1+v2、身份提取、JWKS 预热）。
//!
//! 模式的稳定字符串标识（`auth_mode` 上下文键）+ 是否启用（`auth_enabled`）在
//! `generate_mcp_python_language` 的 `to_template_context()` 之后注入，驱动模板按
//! 鉴权开关分支渲染。镜像 `McpBackend` 的枚举/再导出/上下文注入模式。

use serde::{Deserialize, Serialize};

/// Whether the generated mcp-python project ships optional JWT/JWKS auth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AuthMode {
    /// No auth (default). Renders ZERO auth code/config/deps.
    #[default]
    None,
    /// Unified JWT/JWKS resource-server validation (ADFS / Entra / Okta / OIDC).
    Jwt,
    /// Turnkey Entra ID / Azure AD preset — same JWT/JWKS validation as `Jwt`,
    /// with tenant-derived endpoints, dual v1+v2 issuer, identity extraction, JWKS warm-up.
    AzureAd,
}

impl AuthMode {
    /// 模式的稳定字符串标识（写入模板上下文 `auth_mode` 等）。
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthMode::None => "none",
            AuthMode::Jwt => "jwt",
            AuthMode::AzureAd => "azure-ad",
        }
    }

    /// 是否启用鉴权 —— 模板用 `<%if auth_enabled%>` 分支。
    pub fn is_enabled(&self) -> bool {
        matches!(self, AuthMode::Jwt | AuthMode::AzureAd)
    }

    /// 是否为 Azure AD 预设 —— 模板用 `<%if auth_is_azure_ad%>` 分支。
    pub fn is_azure_ad(&self) -> bool {
        matches!(self, AuthMode::AzureAd)
    }

    /// 从字符串解析鉴权模式；大小写不敏感。
    pub fn parse_from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" => Some(AuthMode::None),
            "jwt" => Some(AuthMode::Jwt),
            "azure-ad" | "azuread" | "azure_ad" => Some(AuthMode::AzureAd),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "auth_options_tests.rs"]
mod tests;
