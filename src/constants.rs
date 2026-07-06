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
