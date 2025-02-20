/** a simple lisp frontend for testing before i write the k and lua */
use crate::{
    Res,
    bc::{Blk, BlkType, Body, Instr, Obj, ObjType, Time},
    err_fmt,
};

macro_rules! push {
    ($v:expr => [ $x:expr ]) => {{
        let i = $v.len();
        $v.push($x);
        i
    }};
    ($v:expr => [ $( $x:expr ),* $(,)* ]) => {{
        let i = $v.len();
        [$($x),*].into_iter().for_each(|x| $v.push(x));
        i
    }};
}

static mut VAR_NUM: u8 = 0;

fn mk_var() -> u8 {
    unsafe {
        let x = VAR_NUM;
        VAR_NUM += 1;
        x
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum LeafType {
    C,
    F,
    M,
    D,
    A,
    Fun,
}

impl Into<ObjType> for LeafType {
    fn into(self) -> ObjType {
        use LeafType::*;
        match self {
            C => ObjType::C,
            F => ObjType::F,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Leaf<'a> {
    X(&'static str),
    F(f64),
    C(char),
    M(&'static str, &'a Leaf<'a>),
    D(&'static str, &'a Leaf<'a>, &'a Leaf<'a>),
    Fun {
        a: Vec<(&'static str, LeafType)>,
        v: Vec<Leaf<'a>>,
    },
}

macro_rules! impl_math {
    ($self:expr, ($x:expr, $y:expr) { $($i:expr),* $(,)* }) => {{
        let i = push!($self.instrs => [
            Instr::Push($x),
            Instr::Push($y),
            $($i)*,
            Instr::Ret
        ]);
        let b = push!($self.bodies => [Body {
            start: i,
            vars: 0,
            names: Vec::new(),
            export: Vec::new(),
        }]);
        push!($self.blocks => [Blk(BlkType::Fun, Time::Immediate, b)])
    }};
}

#[derive(Debug, Clone, PartialEq)]
pub struct Machine<'a> {
    pub vars: Vec<(&'static str, u8, LeafType)>,
    pub instrs: Vec<Instr>,
    pub blocks: Vec<Blk>,
    pub bodies: Vec<Body<'a>>,
}

impl<'a> Machine<'a> {
    pub fn new() -> Self {
        Self {
            vars: Vec::new(),
            instrs: Vec::new(),
            blocks: Vec::new(),
            bodies: Vec::new(),
        }
    }

    pub fn compile(&mut self, l: &'a Leaf<'a>) -> Res<usize> {
        let idx = match l {
            Leaf::F(x) => {
                let i = push!(self.instrs => [
                    Instr::Push(Obj::F(*x)),
                    Instr::Ret
                ]);
                let b = push!(self.bodies => [Body {
                    start: i,
                    vars: 0,
                    names: Vec::new(),
                    export: Vec::new(),
                }]);
                push!(self.blocks => [Blk(BlkType::Fun, Time::Immediate, b)])
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
                push!(self.blocks => [Blk(BlkType::Fun, Time::Immediate, b)])
            }

            Leaf::D("+", Leaf::F(x), Leaf::F(y)) => {
                impl_math!(self, (Obj::F(*x), Obj::F(*y)){Instr::AddF})
            }
            Leaf::D("-", Leaf::F(x), Leaf::F(y)) => {
                impl_math!(self, (Obj::F(*x), Obj::F(*y)){Instr::SubF})
            }
            Leaf::D("*", Leaf::F(x), Leaf::F(y)) => {
                impl_math!(self, (Obj::F(*x), Obj::F(*y)){Instr::MulF})
            }
            Leaf::D("%", Leaf::F(x), Leaf::F(y)) => {
                impl_math!(self, (Obj::F(*x), Obj::F(*y)){Instr::DivF})
            }

            Leaf::D("+", Leaf::F(x), Leaf::X(y)) => {
                let i = push!(self.instrs => [
                    Instr::Push(Obj::F(*x)),
                    Instr::Load(self.vars.iter()
                        .filter(|(n, _, _)| n == y)
                        .map(|(_, i, _)| *i)
                        .last()
                        .expect("{y} undefined in leaf compilation")
                    ),
                    Instr::AddF,
                    Instr::Ret
                ]);
                let b = push!(self.bodies => [Body {
                    start: i,
                    vars: 0,
                    names: Vec::new(),
                    export: Vec::new(),
                }]);
                push!(self.blocks => [Blk(BlkType::Fun, Time::Immediate, b)])
            }

            Leaf::Fun { a, v } => {
                let mut ins = Vec::new();

                a.iter().rev().for_each(|(x, t)| {
                    let i = mk_var();
                    self.vars.push((x, i, *t));
                    ins.push(Instr::Local(i));
                });

                let mut o = None;
                for x in v.iter() {
                    o = Some(self.compile(x)?);
                }

                push!(ins => [
                    o
                        .and_then(|x| Some(Instr::Push(Obj::Fun(x))))
                        .unwrap_or(Instr::Nop),
                    Instr::Apply0,
                    Instr::Ret,
                ]);

                let i = self.instrs.len();
                self.instrs.append(&mut ins);

                let b = push!(self.bodies => [Body {
                    start: i,
                    vars: a.len(),
                    names: a.into_iter()
                        .map(|(n, x)| (*n, (*x).into()))
                        .collect::<Vec<(&str, ObjType)>>(),
                    export: Vec::new(),
                }]);
                let f = push!(self.blocks => [
                    Blk(BlkType::Fun, Time::Deferred, b)
                ]);

                let i = push!(self.instrs => [
                    Instr::Push(Obj::Fun(f)),
                    Instr::Ret
                ]);
                let b = push!(self.bodies => [Body {
                    start: i,
                    vars: 0,
                    names: Vec::new(),
                    export: Vec::new(),
                }]);
                push!(self.blocks => [
                    Blk(BlkType::Fun, Time::Immediate, b)]
                );

                f
            }

            Leaf::D("@", x @ Leaf::Fun { a: _, v: _ }, y) => {
                let f = self.compile(x)?;
                let g = self.compile(y)?;

                let i = push!(self.instrs => [
                    Instr::Push(Obj::Fun(g)),
                    Instr::Apply0,
                    Instr::Push(Obj::Fun(f)),
                    Instr::Swap2,
                    Instr::Apply1,
                    Instr::Ret
                ]);
                let b = push!(self.bodies => [Body {
                    start: i,
                    vars: 0,
                    names: Vec::new(),
                    export: Vec::new(),
                }]);
                push!(self.blocks => [Blk(BlkType::Fun, Time::Immediate, b)])
            }

            x => err_fmt!("cannot compile leaf {x:?}")?,
        };

        Ok(idx)
    }

    fn get_var(&self, x: &str) -> u8 {
        *self
            .vars
            .iter()
            .filter(|(n, _, _)| n == &x)
            .map(|(_, i, _)| i)
            .last()
            .expect("x not defined in x+y")
    }
}
