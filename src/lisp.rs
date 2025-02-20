/** a simple lisp frontend for testing before i write the k and lua */
use crate::{
    Res,
    bc::{Block, BlockType, Body, Instr, Obj, Time},
    err_fmt,
};

macro_rules! push {
    ($v:expr => [ $x:expr ]) => {{
        let i = $v.len();
        $v.push($x);
        i
    }};
    ($v:expr => [ $( $x:expr ),* ]) => {{
        let i = $v.len();
        [$($x),*].into_iter().for_each(|x| $v.push(x));
        i
    }};
}

macro_rules! push_vec {
    ($v:expr => $x:expr) => {{
        let i = $v.len();
        push!($v => [
            Instr::Push(Obj::Sz($x.len())),
            Instr::Break
        ]);
        $v.append($x);
        i
    }};
}

#[derive(Debug, Clone, PartialEq)]
pub enum Leaf<'a> {
    X(&'static str),
    I(i32),
    C(char),
    M(&'static str, &'a Leaf<'a>),
    D(&'static str, &'a Leaf<'a>, &'a Leaf<'a>),
    A(Vec<Leaf<'a>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Machine<'a> {
    pub instrs: Vec<Instr>,
    pub blocks: Vec<Block>,
    pub bodies: Vec<Body<'a>>,
}

impl<'a> Machine<'a> {
    pub fn new() -> Self {
        Self {
            instrs: Vec::new(),
            blocks: Vec::new(),
            bodies: Vec::new(),
        }
    }

    pub fn compile(&mut self, l: &'a Leaf<'a>) -> Res<()> {
        match l {
            Leaf::I(x) => {
                let i = push!(self.instrs => [
                    Instr::Push(Obj::I(*x)),
                    Instr::Ret
                ]);
                let b = push!(self.bodies => [Body {
                    start: i,
                    vars: 0,
                    names: Vec::new(),
                    export: Vec::new(),
                }]);
                self.blocks.push(Block(BlockType::Fun, Time::Immediate, b));
                Ok(())
            }

            Leaf::C(x) => {
                let i = push!(self.instrs => [
                    Instr::Push(Obj::C(*x)),
                    Instr::Ret
                ]);
                let b = push!(self.bodies => [Body {
                    start: i,
                    vars: 0,
                    names: Vec::new(),
                    export: Vec::new(),
                }]);
                self.blocks.push(Block(BlockType::Fun, Time::Immediate, b));
                Ok(())
            }

            Leaf::D("+", Leaf::I(x), Leaf::I(y)) => {
                let i = push!(self.instrs => [
                    Instr::Push(Obj::I(*x)),
                    Instr::Push(Obj::I(*y)),
                    Instr::AddI,
                    Instr::Ret
                ]);
                let b = push!(self.bodies => [Body {
                    start: i,
                    vars: 0,
                    names: Vec::new(),
                    export: Vec::new(),
                }]);
                self.blocks.push(Block(BlockType::Fun, Time::Immediate, b));
                Ok(())
            }

            Leaf::D("-", Leaf::I(x), Leaf::I(y)) => {
                let i = push!(self.instrs => [
                    Instr::Push(Obj::I(*x)),
                    Instr::Push(Obj::I(*y)),
                    Instr::SubI,
                    Instr::Ret
                ]);
                let b = push!(self.bodies => [Body {
                    start: i,
                    vars: 0,
                    names: Vec::new(),
                    export: Vec::new(),
                }]);
                self.blocks.push(Block(BlockType::Fun, Time::Immediate, b));
                Ok(())
            }

            x => err_fmt!("cannot compile leaf {x:?}"),
        }
    }
}
