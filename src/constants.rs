use serde::{Deserialize, Serialize};

/// 模板参数常量定义
///
/// 本文件定义了所有生成器中使用的参数名称常量，
/// 统一使用snake_case命名规范以符合Rust代码风格
/// 支持的编程语言枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Go,
    Python,
    Rust,
    TypeScript,
}

impl Language {
    /// 获取语言的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Go => "Go",
            Language::Python => "Python",
            Language::Rust => "Rust",
            Language::TypeScript => "TypeScript",
        }
    }

    /// 该语言在 `templates/build/<dir>/` 下对应的小写目录名。
    ///
    /// 供 `--with-build` 的项目级渲染步骤把 `Language` 映射到统一的
    /// 构建模板目录（Makefile/Dockerfile）。
    pub fn build_dir(&self) -> &'static str {
        match self {
            Language::Go => "go",
            Language::Python => "python",
            Language::Rust => "rust",
            Language::TypeScript => "typescript",
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 支持的框架枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Framework {
    /// 无框架（纯语言项目）
    None,
    Gin,
    GoZero,
    /// Go MCP server（Gin + go-sdk，streamable-HTTP + SSE，proto-gen-jsonschema 约束）
    McpServer,
    /// Python FastAPI（配置驱动、业务代码集中的脚手架）
    FastApi,
    /// Python MCP server（FastMCP / official mcp SDK，streamable-HTTP + SSE，Pydantic auto-schema）
    McpServerPython,
    Tauri,
    Vue3,
    React,
}

impl Framework {
    /// 获取框架的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            Framework::None => "None",
            Framework::Gin => "Gin",
            Framework::GoZero => "go-zero",
            Framework::McpServer => "mcp-server",
            Framework::FastApi => "fastapi",
            Framework::McpServerPython => "mcp-python",
            Framework::Tauri => "Tauri",
            Framework::Vue3 => "Vue3",
            Framework::React => "React",
        }
    }

    /// 获取框架的显示名称（用于用户界面）
    pub fn display_name(&self) -> &'static str {
        match self {
            Framework::None => "None (Pure Language Project)",
            Framework::Gin => "Gin (Web Framework)",
            Framework::GoZero => "go-zero (Microservice Framework)",
            Framework::McpServer => "MCP Server (Gin + go-sdk, streamable-HTTP + SSE)",
            Framework::FastApi => "FastAPI (Config-driven API Framework)",
            Framework::McpServerPython => "MCP Server — Python (FastMCP, streamable-HTTP + SSE)",
            Framework::Tauri => "Tauri (Desktop App Framework)",
            Framework::Vue3 => "Vue3 (Frontend Framework)",
            Framework::React => "React (Frontend Framework)",
        }
    }

    /// 从字符串解析框架
    pub fn parse_from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" | "" => Some(Framework::None),
            "gin" => Some(Framework::Gin),
            "go-zero" => Some(Framework::GoZero),
            "mcp-server" | "mcp" => Some(Framework::McpServer),
            "fastapi" | "fast-api" => Some(Framework::FastApi),
            "mcp-python" | "mcp-py" => Some(Framework::McpServerPython),
            "tauri" => Some(Framework::Tauri),
            "vue3" | "vue" => Some(Framework::Vue3),
            "react" => Some(Framework::React),
            _ => None,
        }
    }

    /// 获取指定语言支持的所有框架
    pub fn frameworks_for_language(language: Language) -> Vec<Framework> {
        match language {
            Language::Go => vec![Framework::Gin, Framework::GoZero, Framework::McpServer],
            Language::Python => {
                vec![
                    Framework::None,
                    Framework::FastApi,
                    Framework::McpServerPython,
                ]
            }
            Language::Rust => vec![Framework::None, Framework::Tauri],
            Language::TypeScript => vec![Framework::Vue3, Framework::React],
        }
    }
}

impl std::fmt::Display for Framework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// 默认值常量 - 用于各种工具和语言的默认版本配置
pub mod defaults {
    /// Rust 默认版本
    pub const RUST_VERSION: &str = "1.88";
    /// uv 默认版本（Python 项目包管理器）
    pub const UV_VERSION: &str = "0.9.1";
    /// ruff 默认版本（Python linter/formatter）
    pub const RUFF_VERSION: &str = "0.12.1";
    /// Python 最低支持版本
    pub const PYTHON_MIN_VERSION: &str = "3.12";
}

/// 字符串转换工具函数
pub mod string_utils {
    /// 将字符串转换为PascalCase
    pub fn to_pascal_case(s: &str) -> String {
        // 处理连字符、下划线和驼峰命名分隔的单词
        let mut result = String::new();
        let mut capitalize_next = true;

        for ch in s.chars() {
            if ch == '_' || ch == '-' {
                capitalize_next = true;
            } else if ch.is_uppercase() && !result.is_empty() {
                // 如果遇到大写字母且不是第一个字符，说明是驼峰命名
                result.push(ch);
                capitalize_next = false;
            } else if capitalize_next {
                result.push(ch.to_uppercase().next().unwrap_or(ch));
                capitalize_next = false;
            } else {
                result.push(ch.to_lowercase().next().unwrap_or(ch));
            }
        }

        result
    }

    /// 将字符串转换为kebab-case
    /// allow(dead_code): not yet wired into a template helper, but locked by a
    /// unit test (test_to_kebab_case) that is part of the Phase 1 safety net.
    #[allow(dead_code)]
    pub fn to_kebab_case(s: &str) -> String {
        let mut result = String::new();
        let chars = s.chars().peekable();

        for ch in chars {
            if ch.is_uppercase() && !result.is_empty() {
                result.push('-');
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        }

        result
    }

    /// 将字符串转换为snake_case
    pub fn to_snake_case(s: &str) -> String {
        let mut result = String::new();
        let chars = s.chars().peekable();

        for ch in chars {
            if ch.is_uppercase() && !result.is_empty() {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        }

        result
    }

    pub fn to_crate_ident(s: &str) -> String {
        let mut result = String::new();
        let mut previous_was_underscore = false;

        for ch in s.chars() {
            let normalized = if ch.is_ascii_alphanumeric() || ch == '_' {
                ch.to_ascii_lowercase()
            } else {
                '_'
            };

            if normalized == '_' {
                if !previous_was_underscore {
                    result.push(normalized);
                }
                previous_was_underscore = true;
            } else {
                result.push(normalized);
                previous_was_underscore = false;
            }
        }

        if !result
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        {
            result.insert(0, '_');
        }

        result
    }

    /// 将项目名转换为合法的 Protobuf package 标识符。
    pub fn to_proto_ident(s: &str) -> String {
        to_crate_ident(s)
    }

    /// 转换为合法的 Cargo 包名，并为数字开头的名称添加下划线前缀。
    pub fn to_cargo_package_name(s: &str) -> String {
        let mut result: String = s
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                    ch
                } else {
                    '_'
                }
            })
            .collect();

        if result.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
            result.insert(0, '_');
        }

        result
    }

    /// 由项目名派生一个安全、防碰撞的 Docker 镜像名（形如 `<slug>-<digest>:latest`）。
    ///
    /// slug：ASCII 小写；保留 ASCII 字母数字；其它连续字符折叠为单个 `-`；去首尾 `-`；
    /// 全无合法字符时用 `app`；最多 80 个 ASCII 字符。digest：始终追加原始项目名 UTF-8
    /// 字节 SHA-256 的前 24 个十六进制字符——故大小写、`_`/`-`、不同 Unicode、被截断的
    /// 长名都不会折叠到同一镜像名。Docker recipe 与 helper 只消费该键，绝不用原始项目名。
    pub fn to_docker_image_name(project_name: &str) -> String {
        use sha2::{Digest, Sha256};

        let mut slug = String::new();
        let mut previous_was_dash = false;
        for ch in project_name.chars() {
            if ch.is_ascii_alphanumeric() {
                slug.push(ch.to_ascii_lowercase());
                previous_was_dash = false;
            } else if !previous_was_dash {
                slug.push('-');
                previous_was_dash = true;
            }
        }
        let slug = slug.trim_matches('-');
        let mut slug = if slug.is_empty() {
            "app".to_string()
        } else {
            slug.to_string()
        };
        if slug.len() > 80 {
            slug.truncate(80);
            let slug_trimmed = slug.trim_end_matches('-');
            slug = if slug_trimmed.is_empty() {
                "app".to_string()
            } else {
                slug_trimmed.to_string()
            };
        }

        let mut hasher = Sha256::new();
        hasher.update(project_name.as_bytes());
        let digest = hasher.finalize();
        let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
        let short = &hex[..24];

        format!("{slug}-{short}:latest")
    }
}

#[cfg(test)]
mod tests {
    use super::string_utils::*;
    use super::{Framework, Language};

    #[test]
    fn fastapi_parses_round_trips_and_is_listed_for_python() {
        assert_eq!(
            Framework::parse_from_str("fastapi"),
            Some(Framework::FastApi)
        );
        assert_eq!(
            Framework::parse_from_str("FastAPI"),
            Some(Framework::FastApi)
        );
        assert_eq!(Framework::FastApi.as_str(), "fastapi");
        assert_eq!(
            Framework::parse_from_str(Framework::FastApi.as_str()),
            Some(Framework::FastApi)
        );

        let python_frameworks = Framework::frameworks_for_language(Language::Python);
        assert!(python_frameworks.contains(&Framework::FastApi));
        assert!(python_frameworks.contains(&Framework::None));
    }

    #[test]
    fn mcp_server_parses_round_trips_and_is_listed_for_go() {
        assert_eq!(
            Framework::parse_from_str("mcp-server"),
            Some(Framework::McpServer)
        );
        assert_eq!(Framework::parse_from_str("mcp"), Some(Framework::McpServer));
        assert_eq!(Framework::McpServer.as_str(), "mcp-server");
        assert_eq!(
            Framework::parse_from_str(Framework::McpServer.as_str()),
            Some(Framework::McpServer)
        );

        let go_frameworks = Framework::frameworks_for_language(Language::Go);
        assert!(go_frameworks.contains(&Framework::McpServer));
        assert!(go_frameworks.contains(&Framework::Gin));
        assert!(go_frameworks.contains(&Framework::GoZero));
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("test_project"), "TestProject");
        assert_eq!(to_pascal_case("single"), "Single");
    }

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case("HelloWorld"), "hello-world");
        assert_eq!(to_kebab_case("TestProject"), "test-project");
        assert_eq!(to_kebab_case("single"), "single");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("TestProject"), "test_project");
        assert_eq!(to_snake_case("single"), "single");
    }

    #[test]
    fn test_to_crate_ident_sanitizes_project_names() {
        assert_eq!(to_crate_ident("My-Cool_App"), "my_cool_app");
        assert_eq!(to_crate_ident("123-app"), "_123_app");
        assert_eq!(to_crate_ident("my.app"), "my_app");
    }

    #[test]
    fn proto_ident_sanitizes_project_names() {
        assert_eq!(to_proto_ident("service-desk"), "service_desk");
        assert_eq!(to_proto_ident("123-app"), "_123_app");
        assert_eq!(to_proto_ident("my.app"), "my_app");
        assert_eq!(to_proto_ident("clean_name"), "clean_name");
    }

    #[test]
    fn cargo_package_name_preserves_legal_names_and_prefixes_leading_digits() {
        assert_eq!(to_cargo_package_name("123-app"), "_123-app");
        assert_eq!(to_cargo_package_name("my-app"), "my-app");
        assert_eq!(to_cargo_package_name("My_App"), "My_App");
        assert_eq!(to_cargo_package_name("my.app"), "my_app");
    }

    #[test]
    fn docker_image_name_is_lowercase_slug_plus_digest_and_collision_resistant() {
        // Slug + 24-hex digest suffix, lowercased, non-alnum folded to single '-'.
        let a = to_docker_image_name("OrderApi");
        assert!(a.starts_with("orderapi-"), "readable slug preserved: {a}");
        assert!(a.ends_with(":latest"), "tag appended: {a}");

        // Near-collision inputs must NOT fold to the same image name.
        let foo_us = to_docker_image_name("foo_bar");
        let foo_dash = to_docker_image_name("foo-bar");
        assert_ne!(
            foo_us, foo_dash,
            "distinct originals must differ: {foo_us} vs {foo_dash}"
        );

        // Case differences differ (digest of original bytes).
        assert_ne!(to_docker_image_name("Foo"), to_docker_image_name("foo"));

        // Distinct Unicode originals differ even if slug collapses.
        assert_ne!(to_docker_image_name("café"), to_docker_image_name("cafe"));

        // Empty-of-legal-chars falls back to `app` slug but stays unique per original.
        let dots = to_docker_image_name("...");
        assert!(
            dots.starts_with("app-"),
            "empty slug falls back to app: {dots}"
        );
        assert_ne!(dots, to_docker_image_name("///"));

        // Deterministic.
        assert_eq!(
            to_docker_image_name("repeatable"),
            to_docker_image_name("repeatable")
        );
    }

    #[test]
    fn docker_image_name_slug_is_length_bounded_and_distinct_on_truncation() {
        let long_a = "x".repeat(200);
        let long_b = format!("{}y", "x".repeat(199));
        let img_a = to_docker_image_name(&long_a);
        let img_b = to_docker_image_name(&long_b);

        // Slug is capped: everything before the last '-<digest>:latest' is <= 80 ASCII chars.
        let slug_a = img_a
            .strip_suffix(":latest")
            .and_then(|s| s.rsplit_once('-'))
            .map(|(slug, _)| slug)
            .expect("slug segment");
        assert!(
            slug_a.len() <= 80,
            "slug must be length-bounded: {} chars",
            slug_a.len()
        );

        // Two long names sharing a truncated slug still differ via the digest.
        assert_ne!(
            img_a, img_b,
            "truncated long names must differ: {img_a} vs {img_b}"
        );
    }

    #[test]
    fn mcp_server_python_parses_round_trips_and_is_listed_for_python() {
        assert_eq!(
            Framework::parse_from_str("mcp-python"),
            Some(Framework::McpServerPython)
        );
        assert_eq!(
            Framework::parse_from_str("mcp-py"),
            Some(Framework::McpServerPython)
        );
        assert_eq!(
            Framework::parse_from_str("MCP-Python"),
            Some(Framework::McpServerPython)
        );
        assert_eq!(Framework::McpServerPython.as_str(), "mcp-python");
        assert_eq!(
            Framework::parse_from_str(Framework::McpServerPython.as_str()),
            Some(Framework::McpServerPython)
        );

        let python_frameworks = Framework::frameworks_for_language(Language::Python);
        assert!(python_frameworks.contains(&Framework::McpServerPython));
        assert!(python_frameworks.contains(&Framework::FastApi));
        assert!(python_frameworks.contains(&Framework::None));
    }
}
