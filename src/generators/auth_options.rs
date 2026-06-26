//! mcp-python 鉴权模式选择 —— `AuthMode` 枚举。
//!
//! 决定生成的 mcp-python 项目是否内置可选的 JWT/JWKS 资源服务器鉴权：
//! - `None`：不启用（默认），渲染零鉴权代码 —— 与历史输出字节一致。
//! - `Jwt`：统一 JWT/JWKS 校验路径，覆盖 AD FS / Entra ID / Okta / 通用 OIDC。
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
}

impl AuthMode {
    /// 模式的稳定字符串标识（写入模板上下文 `auth_mode` 等）。
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthMode::None => "none",
            AuthMode::Jwt => "jwt",
        }
    }

    /// 是否启用鉴权 —— 模板用 `<%if auth_enabled%>` 分支。
    pub fn is_enabled(&self) -> bool {
        matches!(self, AuthMode::Jwt)
    }

    /// 从字符串解析鉴权模式；大小写不敏感。
    pub fn parse_from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" => Some(AuthMode::None),
            "jwt" => Some(AuthMode::Jwt),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_round_trip_for_both_modes() {
        for mode in [AuthMode::None, AuthMode::Jwt] {
            let parsed = AuthMode::parse_from_str(mode.as_str());
            assert_eq!(parsed, Some(mode));
        }
    }

    #[test]
    fn as_str_values() {
        assert_eq!(AuthMode::None.as_str(), "none");
        assert_eq!(AuthMode::Jwt.as_str(), "jwt");
    }

    #[test]
    fn is_enabled_flag() {
        assert!(AuthMode::Jwt.is_enabled());
        assert!(!AuthMode::None.is_enabled());
    }

    #[test]
    fn default_is_none() {
        assert_eq!(AuthMode::default(), AuthMode::None);
    }

    #[test]
    fn parse_is_case_insensitive() {
        assert_eq!(AuthMode::parse_from_str("JWT"), Some(AuthMode::Jwt));
        assert_eq!(AuthMode::parse_from_str("None"), Some(AuthMode::None));
    }

    #[test]
    fn parse_bogus_is_none() {
        assert!(AuthMode::parse_from_str("bogus").is_none());
    }
}
