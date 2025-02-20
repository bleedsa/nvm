use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Obj {
    I(i32),
    C(char),
    F(f64),
    Fun(usize),
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum ObjType {
    I,
    C,
    F,
    Fun,
}

impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Obj::I(x) => x.to_string(),
            Obj::C(c) => c.to_string(),
            Obj::F(x) => x.to_string(),
            Obj::Fun(x) => format!("{{&{x}}}"),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Instr {
    /* variables */
    Push(Obj),
    Pop,
    Local(u8),
    Load(u8),

    /* math */
    AddF,
    SubF,
    MulF,
    DivF,

    /* functions */
    Apply0,
    Apply1,
    ApplyN,

    /* stack */
    Swap2,

    /* control */
    Ret,
    Break,
    Nop,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum BlkType {
    Fun,
    One,
    Two,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum Time {
    Immediate,
    Deferred,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Blk(pub BlkType, pub Time, pub usize);

impl Blk {
    #[inline]
    pub fn idx(&self) -> usize {
        self.2
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Body<'a> {
    pub start: usize,
    pub vars: usize,
    pub names: Vec<(&'a str, ObjType)>,
    pub export: Vec<bool>,
}

impl<'a> Body<'a> {
    pub fn exported(&self) -> Vec<(&'a str, ObjType)> {
        assert_eq!(self.names.len(), self.export.len());
        self.names
            .iter()
            .enumerate()
            .filter(|(i, _)| self.export[*i])
            .map(|(_, x)| *x)
            .collect::<Vec<_>>()
    }
}
