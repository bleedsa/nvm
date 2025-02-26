/** a simple lisp frontend for testing before i write the k and lua */
use crate::{
    BodyIterator, Res,
    bc::{Blk, BlkType, Body, Instr, Obj, ObjType, Time},
    err_fmt,
};
use std::collections::HashMap;

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

macro_rules! set {
    ($v:expr => [ $x:expr ]) => {{
        let i = $v.len();
        $v.insert($v.len(), $x);
        i
    }};
    ($v:expr => [ $($x:expr),* $(,)* ]) => {{
        let i = $v.len();
        $(
            if !$v.contains($x) {
                $v.insert($v.len(), $x);
                i
            } else {
                $v.find(|x| x == $x).unwrap()
            }
        )*
        i
    }};
}

macro_rules! impl_math {
    ($self:expr, ($x:expr, $y:expr) { $($i:expr),* $(,)* }) => {{
        let i = push!($self.instrs => [
            Instr::Push($x),
            Instr::Push($y),
            $($i),*,
            Instr::Ret
        ]);
        let b = set!($self.bodies => [Body {
            start: i,
            vars: 0,
            names: Vec::new(),
            export: Vec::new(),
        }]);
        set!($self.blocks => [Blk(BlkType::Fun, Time::Immediate, b)])
    }};
}

macro_rules! impl_named_math {
    ($self:expr, var, $x:expr) => {{
        Instr::Load($self.get_var_by_name($x))
    }};
    ($self:expr, flt, $x:expr) => {{
        Instr::Push(Obj::F(*$x))
    }};
    ($self:expr, ($(($t:tt $y:ident)),*) { $($i:expr),* $(,)* }) => {{
        let i = push!($self.instrs => [
            $( impl_named_math!($self, $t, $y) ),*,
            $($i),*,
            Instr::Ret,
        ]);
        let b = set!($self.bodies => [Body {
            start: i,
            vars: 0,
            names: Vec::new(),
            export: Vec::new(),
        }]);
        set!($self.blocks => [Blk(BlkType::Fun, Time::Immediate, b)])
    }};
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
    A(Vec<Leaf<'a>>),

    M(&'static str, &'a Leaf<'a>),
    D(&'static str, &'a Leaf<'a>, &'a Leaf<'a>),
    Fun {
        a: Vec<(&'static str, LeafType)>,
        v: Vec<Leaf<'a>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Machine<'a> {
    pub vars: HashMap<usize, (&'static str, LeafType)>,
    pub varn: usize,
    pub labeln: usize,
    pub instrs: Vec<Instr>,
    pub blocks: Vec<Blk>,
    pub bodies: Vec<Body<'a>>,
}

impl<'a> Machine<'a> {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            varn: 0,
            labeln: 0,
            instrs: Vec::new(),
            blocks: Vec::new(),
            bodies: Vec::new(),
        }
    }

    #[inline]
    pub fn iter_body(&self, i: usize) -> BodyIterator {
        BodyIterator::from(&self.instrs, &self.bodies[i])
    }

    #[inline]
    fn add_var(&mut self, n: &'static str, t: LeafType) -> usize {
        let i = self.varn;
        self.vars.insert(i, (n, t));
        self.varn += 1;
        i
    }

    #[inline]
    fn mk_var(&mut self) -> usize {
        let i = self.varn;
        self.varn += 1;
        i
    }

    #[allow(unused)]
    #[cfg(debug_assertions)]
    fn get_var(&self, x: usize) -> usize {
        *self
            .vars
            .iter()
            .filter(|(i, _)| i == &&x)
            .last()
            .map(|(i, _)| i)
            .expect(&format!("{x} not defined"))
    }

    fn get_var_by_name(&self, x: &str) -> usize {
        self.vars
            .iter()
            .filter(|(_, (n, _))| &x == n)
            .last()
            .map(|(i, _)| *i)
            .expect(&format!("variable {x} undefined"))
    }

    #[inline]
    fn mk_label(&mut self) -> usize {
        let x = self.labeln;
        self.labeln += 1;
        x
    }

    /*
     * ================================================
     * VERB IMPLS
     * =================================================
     */

    /** iota !x: push (0..x) */
    fn iota(&mut self, x: usize) -> Res<usize> {
        let i = self.instrs.len();

        /* push 0..x */
        (0..x).for_each(|x| {
            self.instrs.push(Instr::Push(Obj::F(x as f64)));
        });
        push!(self.instrs => [
            Instr::Vec(x),
            Instr::Ret,
        ]);

        let b = set!(self.bodies => [Body {
            start: i,
            vars: 0,
            names: Vec::new(),
            export: Vec::new(),
        }]);
        Ok(set!(self.blocks => [Blk(BlkType::Fun, Time::Immediate, b)]))
    }

    /** apply1 x@y: apply y to x */
    fn apply1(&mut self, x: &'a Leaf<'a>, y: &'a Leaf<'a>) -> Res<usize> {
        let f = self.compile(x)?;
        let g = self.compile(y)?;
        Ok(self.immediate(&[
            Instr::Push(Obj::Fun(f)),
            Instr::Push(Obj::Fun(g)),
            Instr::Apply0,
            Instr::Apply1,
        ]))
    }

    /** applyn x.y: push x applied to each y */
    fn apply_n(&mut self, x: &'a Leaf<'a>, y: &'a [Leaf<'a>]) -> Res<usize> {
        let f = self.compile(x)?;
        let mut ins = Vec::new();

        /* we split off and compile each object in y while at the
         * same time writing Jmp instrs to the ins vec, followed
         * by the function and application */
        for i in y.into_iter() {
            let a = self.compile(i)?;
            ins.push(Instr::Jmp(a));
        }
        push!(ins => [
            Instr::Push(Obj::Fun(f)),
            Instr::ApplyN,
            Instr::Ret,
        ]);

        let i = self.instrs.len();
        self.instrs.append(&mut ins);
        let b = set!(self.bodies => [Body {
            start: i,
            vars: 0,
            names: Vec::new(),
            export: Vec::new(),
        }]);
        Ok(set!(self.blocks => [Blk(BlkType::Fun, Time::Immediate, b)]))
    }

    fn apply_n_to(&mut self, x: &'a Leaf<'a>, y: &'a Leaf<'a>) -> Res<usize> {
        let m = self.compile(y)?;
        let f = self.compile(x)?;
        Ok(self.immediate(&[
            Instr::Jmp(m),
            Instr::PopVec,
            Instr::Push(Obj::Fun(f)),
            Instr::ApplyN,
        ]))
    }

    fn negate(&mut self, x: f64) -> Res<usize> {
        Ok(self.immediate(&[Instr::Push(Obj::F(-x))]))
    }

    fn immediate(&mut self, v: &[Instr]) -> usize {
        let i = self.instrs.len();
        v.iter().for_each(|x| self.instrs.push(*x));
        self.instrs.push(Instr::Ret);

        let b = set!(self.bodies => [Body {
            start: i,
            vars: 0,
            names: Vec::new(),
            export: Vec::new(),
        }]);
        set!(self.blocks => [Blk(BlkType::Fun, Time::Immediate, b)])
    }

    fn lambda(&mut self, v: &[Instr]) -> usize {
        let i = self.instrs.len();
        v.iter().for_each(|x| self.instrs.push(*x));
        push!(self.instrs => [
            Instr::Ret,
        ]);

        let b = set!(self.bodies => [Body {
            start: i,
            vars: 0,
            names: Vec::new(),
            export: Vec::new(),
        }]);
        set!(self.blocks => [Blk(BlkType::Fun, Time::Immediate, b)])
    }

    pub fn compile(&mut self, l: &'a Leaf<'a>) -> Res<usize> {
        /* this giant mangled match statement compiles a block and
         * returns the index. then we just return it */
        let idx = match l {
            Leaf::X(x) => {
                let i = self.get_var_by_name(x);
                self.immediate(&[Instr::Load(i)])
            }

            Leaf::F(x) => self.immediate(&[Instr::Push(Obj::F(*x))]),

            Leaf::C(x) => self.immediate(&[Instr::Push(Obj::C(*x))]),

            Leaf::A(x) => {
                let mut v = Vec::new();
                for x in x.iter() {
                    v.push(self.compile(x)?);
                }

                let i = self.instrs.len();
                v.iter().for_each(|x| {
                    self.instrs.push(Instr::Jmp(*x));
                });
                push!(self.instrs => [
                    Instr::Vec(v.len()),
                    Instr::Ret,
                ]);

                let b = set!(self.bodies => [Body {
                    start: i,
                    vars: 0,
                    names: Vec::new(),
                    export: Vec::new(),
                }]);
                set!(self.blocks => [Blk(BlkType::Fun, Time::Immediate, b)])
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
                impl_named_math!(self, ((flt x), (var y)){Instr::AddF})
            }
            Leaf::D("+", Leaf::X(x), Leaf::F(y)) => {
                impl_named_math!(self, ((var x), (flt y)){Instr::AddF})
            }
            Leaf::D("+", Leaf::X(x), Leaf::X(y)) => {
                impl_named_math!(self, ((var x), (var y)){Instr::AddF})
            }

            Leaf::Fun { a, v } => {
                /* we'll write instrs to this vec for now so we can compile
                 * the other statements first */
                let mut ins = Vec::new();

                a.iter().rev().for_each(|(x, t)| {
                    let i = self.add_var(x, *t);
                    ins.push(Instr::Local(i));
                });

                let mut it = v.iter();
                let addr = if let Some(x) = it.next() {
                    self.compile(x)?
                } else {
                    err_fmt!("no leaves in fun")?
                };
                for x in it {
                    self.compile(x)?;
                }

                let i = self.instrs.len();
                self.instrs.append(&mut ins);

                push!(self.instrs => [
                    Instr::Jmp(addr),
                    Instr::Ret,
                ]);

                let b = set!(self.bodies => [Body {
                    start: i,
                    vars: a.len(),
                    names: a.into_iter()
                        .map(|(n, x)| (*n, (*x).into()))
                        .collect::<Vec<(&str, ObjType)>>(),
                    export: Vec::new(),
                }]);
                set!(self.blocks => [Blk(BlkType::Fun, Time::Deferred, b)])
            }

            Leaf::D("@", x @ Leaf::Fun { a: _, v: _ }, y) => {
                self.apply1(x, y)?
            }

            Leaf::D(".", x @ Leaf::Fun { a: _, v: _ }, Leaf::A(y)) => {
                self.apply_n(x, &y)?
            }
            Leaf::D(
                ".",
                x @ Leaf::Fun { a: _, v: _ },
                y @ (Leaf::M(_, _) | Leaf::D(_, _, _)),
            ) => self.apply_n_to(x, y)?,

            Leaf::M("-", Leaf::F(x)) => self.negate(*x)?,
            Leaf::M("-", x) => {
                let x = self.compile(x)?;
                self.immediate(&[Instr::Jmp(x), Instr::NegF])
            }

            Leaf::M("!", Leaf::F(x)) => self.iota(*x as usize)?,
            Leaf::M("!", x) => {
                let x = self.compile(x)?;
                let l = self.mk_label();
                let r = self.mk_var();

                let i = push!(self.instrs => [
                    /* jump to x, subtract one. this is the counter */
                    Instr::Jmp(x),
                    Instr::Push(Obj::F(1.)),
                    Instr::SubF,

                    /* make a singleton vector containing 0 */
                    Instr::Push(Obj::F(0.)),
                    Instr::Vec(1),

                    /* start a loop */
                    Instr::Local(r),
                    Instr::Label(l),
                    Instr::Dup,
                    /* add 1 to the tail of the vec */
                    Instr::VecLast(r),
                    Instr::Push(Obj::F(1.)),
                    Instr::AddF,
                    Instr::Dup,
                    /* push the result */
                    Instr::VecPush(r),
                    /* now compare the result to the counter */
                    Instr::CmpF,
                    /* restart the loop if neq */
                    Instr::LJmpNZ(l),

                    /* return the vector */
                    Instr::Load(r),
                ]);
                let b = set!(self.bodies => [Body {
                    start: i,
                    vars: 0,
                    names: Vec::new(),
                    export: Vec::new(),
                }]);
                set!(self.blocks => [Blk(BlkType::Fun, Time::Immediate, b)])
            }

            x => err_fmt!("cannot compile leaf {x:?}")?,
        };

        Ok(idx)
    }

    pub fn dump(&self) {
        println!("instrs:");
        self.instrs
            .iter()
            .enumerate()
            .for_each(|(i, x)| println!("{i:3}: {x:?}"));
        println!("bodies:");
        self.bodies.iter().enumerate().for_each(|(i, x)| {
            println!("{i:3}: {x:?}");
            self.iter_body(i).for_each(|x| println!("    > {x:?}"));
        });
        println!("blocks:");
        self.blocks
            .iter()
            .enumerate()
            .for_each(|(i, x)| println!("{i:3}: {x:?}"));
        println!("vars:");
        self.vars
            .iter()
            .for_each(|(n, (i, x))| println!("{i:3}: {n} = {x:?}"));
    }
}
