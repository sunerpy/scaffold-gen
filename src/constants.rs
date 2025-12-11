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

    /// 获取语言的小写字符串表示
    #[allow(dead_code)]
    pub fn as_lowercase(&self) -> &'static str {
        match self {
            Language::Go => "go",
            Language::Python => "python",
            Language::Rust => "rust",
            Language::TypeScript => "typescript",
        }
    }

    /// 从字符串解析语言
    #[allow(dead_code)]
    pub fn parse_from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "go" => Some(Language::Go),
            "python" => Some(Language::Python),
            "rust" => Some(Language::Rust),
            "typescript" | "ts" => Some(Language::TypeScript),
            _ => None,
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
    Tauri,
    Vue3,
    React,
}

impl Framework {
    /// 获取框架的字符串表示
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Framework::None => "None",
            Framework::Gin => "Gin",
            Framework::GoZero => "go-zero",
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
            Framework::Tauri => "Tauri (Desktop App Framework)",
            Framework::Vue3 => "Vue3 (Frontend Framework)",
            Framework::React => "React (Frontend Framework)",
        }
    }

    /// 获取框架的小写字符串表示
    #[allow(dead_code)]
    pub fn as_lowercase(&self) -> &'static str {
        match self {
            Framework::None => "none",
            Framework::Gin => "gin",
            Framework::GoZero => "go-zero",
            Framework::Tauri => "tauri",
            Framework::Vue3 => "vue3",
            Framework::React => "react",
        }
    }

    /// 从字符串解析框架
    pub fn parse_from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" | "" => Some(Framework::None),
            "gin" => Some(Framework::Gin),
            "go-zero" => Some(Framework::GoZero),
            "tauri" => Some(Framework::Tauri),
            "vue3" | "vue" => Some(Framework::Vue3),
            "react" => Some(Framework::React),
            _ => None,
        }
    }

    /// 获取框架对应的语言
    #[allow(dead_code)]
    pub fn language(&self) -> Option<Language> {
        match self {
            Framework::None => None,
            Framework::Gin => Some(Language::Go),
            Framework::GoZero => Some(Language::Go),
            Framework::Tauri => Some(Language::Rust),
            Framework::Vue3 => Some(Language::TypeScript),
            Framework::React => Some(Language::TypeScript),
        }
    }

    /// 检查是否为无框架
    #[allow(dead_code)]
    pub fn is_none(&self) -> bool {
        matches!(self, Framework::None)
    }

    /// 获取指定语言支持的所有框架
    pub fn frameworks_for_language(language: Language) -> Vec<Framework> {
        match language {
            Language::Go => vec![Framework::Gin, Framework::GoZero],
            Language::Python => vec![], // Python 目前没有框架选项
            Language::Rust => vec![Framework::None, Framework::Tauri],
            Language::TypeScript => vec![Framework::Vue3, Framework::React],
        }
    }

    /// 获取所有框架
    #[allow(dead_code)]
    pub fn all() -> Vec<Framework> {
        vec![
            Framework::None,
            Framework::Gin,
            Framework::GoZero,
            Framework::Tauri,
            Framework::Vue3,
            Framework::React,
        ]
    }
}

impl std::fmt::Display for Framework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// 模板变量名常量 - 用于模板渲染时的变量替换
#[allow(dead_code)]
pub const PROJECT_NAME: &str = "project_name";
#[allow(dead_code)]
pub const FRAMEWORK: &str = "framework";
#[allow(dead_code)]
pub const GO_VERSION: &str = "go_version";

// 网络配置常量 - 预留用于后期扩展的网络配置功能
#[allow(dead_code)]
pub const DEFAULT_HOST: &str = "default_host";
#[allow(dead_code)]
pub const DEFAULT_PORT: &str = "default_port";
#[allow(dead_code)]
pub const HOST: &str = "host";
#[allow(dead_code)]
pub const PORT: &str = "port";
// API 端口配置 - 预留用于后期扩展的 API 服务配置
#[allow(dead_code)]
pub const API_PORT: &str = "api_port";
// RPC 端口配置 - 预留用于后期扩展的 RPC 服务配置
#[allow(dead_code)]
pub const RPC_PORT: &str = "rpc_port";

// 默认值常量 - 用于各种工具和语言的默认版本配置
#[allow(dead_code)]
pub mod defaults {
    // ===== 语言版本 =====
    /// Go 默认版本
    pub const GO_VERSION: &str = "1.24";
    /// Python 默认版本
    pub const PYTHON_VERSION: &str = "3.12";
    /// Rust 默认版本
    pub const RUST_VERSION: &str = "1.75";
    /// Node.js 默认版本
    pub const NODE_VERSION: &str = "20";
    /// TypeScript 默认版本
    pub const TYPESCRIPT_VERSION: &str = "5.0";

    // ===== 工具版本 =====
    /// uv 默认版本
    pub const UV_VERSION: &str = "0.9.5";

    // ===== 网络配置 =====
    /// 默认主机地址
    pub const HOST: &str = "0.0.0.0";
    /// 默认 HTTP 端口
    pub const PORT: i32 = 8080;
    /// API 服务端口
    pub const API_PORT: i32 = 8888;
    /// RPC 服务端口
    pub const RPC_PORT: i32 = 9999;
    /// Vue3/React 开发服务器端口
    pub const VITE_PORT: i32 = 5173;
    /// Tauri 开发服务器端口
    pub const TAURI_PORT: i32 = 1420;
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

    /// 将字符串转换为kebab-case (预留给未来的模板渲染功能)
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
}
