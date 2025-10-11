// 生成器模块
pub mod core;
pub mod framework;
pub mod language;
pub mod orchestrator;
pub mod project;

// 重新导出核心类型 - 使用具体导出避免命名冲突
#[allow(unused_imports)]
pub use core::{Generator, ParameterBuilder, TemplateProcessor};
#[allow(unused_imports)]
pub use project::ProjectGenerator;

// 语言生成器 - 具体导出避免冲突
#[allow(unused_imports)]
pub use language::go::{GoGenerator, GoParams};

// 框架生成器 - 具体导出避免冲突
#[allow(unused_imports)]
pub use framework::gin::{GinGenerator, GinParams};
#[allow(unused_imports)]
pub use framework::go_zero::{GoZeroGenerator, GoZeroParams};

pub use orchestrator::*;
