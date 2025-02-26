#![feature(let_chains)]

pub mod bc;
pub mod lisp;
pub mod vm;

use crate::bc::{Body, Instr};

pub type Res<T> = Result<T, String>;

#[macro_export]
macro_rules! err_fmt {
    ($($x:tt)*) => {{
        Err(format!($($x)*))
    }};
}

/** format! but with colors */
#[macro_export]
macro_rules! fmt {
    (green $($t:tt)*) => {{
        format!("\x1b[0;32m{}\x1b[0;0m", format!($($t)*))
    }};
    ($($t:tt)*) => {{
        format!($($t)*)
    }};
}

#[macro_export]
macro_rules! dbgln {
    (alert, $($t:tt)*) => {{
        #[cfg(test)]
        println!("\x1b[0;34m{}\x1b[0;0m", format!($($t)*));
    }};
    ($($t:tt)*) => {{
        #[cfg(test)]
        println!($($t)*);
    }};
}

#[macro_export]
macro_rules! fatal {
    ($($x:tt)*) => {{
        eprintln!("fatal error: {}", format!($($x)*));
        std::process::exit(-1);
    }};
}

#[macro_export]
macro_rules! heredoc {
    ($($t:tt)*) => {{
        format!($($t)*)
            .split('\n')
            .map(|x| x.trim().to_string() + "\n")
            .collect::<String>()
            .trim()
            .to_string()
    }};
}

pub fn or_fatal<X, Y>(x: Result<X, Y>) -> X
where
    Y: std::fmt::Display,
{
    match x {
        Ok(x) => x,
        Err(e) => fatal!("{e}"),
    }
}

pub struct BodyIterator<'a> {
    pub i: std::slice::Iter<'a, Instr>,
}

impl<'a> BodyIterator<'a> {
    pub fn from(src: &'a [Instr], b: &Body) -> Self {
        Self {
            i: (&src[b.start..]).iter(),
        }
    }
}

impl<'a> Iterator for BodyIterator<'a> {
    type Item = &'a Instr;

    fn next(&mut self) -> Option<Self::Item> {
        self.i
            .next()
            .and_then(|x| if x == &Instr::Ret { None } else { Some(x) })
    }
}

/*
pub mod clr {
    #[derive(Copy, Clone, Debug, PartialEq)]
    #[repr(usize)]
    pub enum Clr {
        Clear = 0,
        Green = 1,
        Blue = 2,
    }

    pub static COLORS: &'static [&'static str] = &[
        "\x1b[0;0m",  /* clear */
        "\x1b[0;32m", /* green */
        "\x1b[0;34m", /* blue */
    ];

    impl Into<&'static str> for Clr {
        #[inline]
        fn into(self) -> &'static str {
            COLORS[self as usize]
        }
    }

    impl Clr {
        #[inline]
        pub fn as_str(self) -> &'static str {
            self.into()
        }
    }
}
*/
