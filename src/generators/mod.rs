// 生成器模块
pub mod auth_options;
pub mod core;
pub mod external;
pub mod framework;
pub mod gin_options;
pub mod language;
pub mod mcp_options;
pub mod orchestrator;
pub mod project;
pub mod registry;

// 重新导出核心类型

// 语言生成器

// 框架生成器

// 编排器
pub use auth_options::AuthMode;
pub use gin_options::GinProjectOptions;
pub use mcp_options::McpBackend;
pub use orchestrator::GeneratorOrchestrator;
