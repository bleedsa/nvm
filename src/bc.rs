use std::fmt;

macro_rules! impl_obj_enum {
    (($obj:ident, $objtype:ident) => { $( $n:ident($($t:ty),*) ),* $(,)* }) => {
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub enum $obj {
            $($n($($t),*)),*
        }

        #[derive(Debug, Copy, Clone, PartialEq)]
        #[repr(u8)]
        pub enum $objtype {
            $($n),*
        }
    }
}

impl_obj_enum!((Obj, ObjType) => {
    C(char),
    F(f64),
    U(usize),
    T(usize),
    Fun(usize),
});

impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Obj::C(c) => c.to_string(),
            Obj::U(x) => x.to_string(),
            Obj::F(x) => x.to_string(),
            Obj::T(x) => format!("[&{x}]"),
            Obj::Fun(x) => format!("{{&{x}}}"),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Instr {
    /* variables */
    Push(Obj),
    Pop,
    /** pop and create variable x */
    Local(usize),
    /** push value loaded from x */
    Load(usize),
    /** create a new label */
    Label(usize),

    /* math */
    AddF,
    SubF,
    MulF,
    DivF,
    NegF,
    CmpF,

    /* vectors & tables */
    /** pop x and make a table */
    Table(usize),
    /** pop x and make a vector */
    Vec(usize),
    /** pop all and make a vector */
    VecFull,
    /** pop a value and push it to a vector */
    VecPush(usize),
    /** push the last value in a vec */
    VecLast(usize),
    /** pop a value off a vec */
    VecPop(usize),

    /* functions */
    Apply0,
    Apply1,
    ApplyN,

    /* stack */
    Dup,
    Swap2,
    /** pop a vec, then pop each item from that vec */
    PopVec,

    /* control */
    Ret,
    Break,
    Nop,
    /** jump to immediate block x */
    Jmp(usize),
    /** pop and jump if zero */
    JmpZ(usize),
    /** jump to a label if zero */
    LJmpZ(usize),
    /** jump to a label if not zero */
    LJmpNZ(usize),
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
