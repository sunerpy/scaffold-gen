pub mod gin;
pub mod go_zero;
// pub mod echo;
// pub mod fiber;

// 具体导出避免命名冲突
#[allow(unused_imports)]
pub use gin::{GinGenerator, GinParams};
#[allow(unused_imports)]
pub use go_zero::{GoZeroGenerator, GoZeroParams};
// pub use echo::*;
// pub use fiber::*;
