pub mod gin;
pub mod go_zero;

// 明确导出各框架生成器和参数类型
pub use gin::{GinGenerator, GinParams};
pub use go_zero::{GoZeroGenerator, GoZeroParams};
