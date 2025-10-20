pub mod go;
pub mod python;

// 明确导出各语言生成器和参数类型
pub use go::{GoGenerator, GoParams};
// Python模块暂时没有完整实现，先不导出
// pub use python::{PythonGenerator, PythonParams};
