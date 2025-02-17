use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Obj {
    I(i32),
    C(char),
}

impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Obj::I(x) => x.to_string(),
            Obj::C(c) => c.to_string(),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Instr {
    Push(Obj),
    Ret,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum BlockType {
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
pub struct Block(pub BlockType, pub Time, pub usize);

impl Block {
    #[inline]
    pub fn idx(&self) -> usize {
        self.2
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Body<'a> {
    pub start: usize,
    pub vars: u64,
    pub names: Vec<&'a str>,
    pub export: Vec<bool>,
}

impl<'a> Body<'a> {
    pub fn exported(&self) -> Vec<&'a str> {
        assert_eq!(self.names.len(), self.export.len());
        self.names
            .iter()
            .enumerate()
            .filter(|(i, _)| self.export[*i])
            .map(|(_, x)| *x)
            .collect::<Vec<_>>()
    }
}
