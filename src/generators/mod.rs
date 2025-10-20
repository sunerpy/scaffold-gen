// 生成器模块
pub mod core;
pub mod framework;
pub mod language;
pub mod orchestrator;
pub mod project;

// 重新导出核心类型
pub use core::{Generator, ParameterBuilder, TemplateProcessor};
pub use project::{ProjectGenerator, ProjectParams};

// 语言生成器
pub use language::go::{GoGenerator, GoParams};

// 框架生成器
pub use framework::gin::{GinGenerator, GinParams};
pub use framework::go_zero::{GoZeroGenerator, GoZeroParams};

// 编排器
pub use orchestrator::{GeneratorOrchestrator, GinProjectOptions};
