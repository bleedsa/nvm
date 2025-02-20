pub mod bc;
pub mod lisp;
pub mod vm;

pub type Res<T> = Result<T, String>;

#[macro_export]
macro_rules! err_fmt {
    ($($x:tt)*) => {{
        Err(format!($($x)*))
    }};
}
