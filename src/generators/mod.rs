// 生成器模块
pub mod core;
pub mod framework;
pub mod language;
pub mod orchestrator;
pub mod project;

// 重新导出核心类型

// 语言生成器

// 框架生成器

// 编排器
pub use orchestrator::{GeneratorOrchestrator, GinProjectOptions};
