use crate::{
    BodyIterator, Res,
    bc::{Blk, BlkType, Body, Instr, Obj, Time},
    dbgln, err_fmt, heredoc,
};
use std::collections::HashMap;

#[cfg(test)]
use colored::Colorize;

#[cfg(test)]
use pad::{Alignment, PadStr};

macro_rules! pop_assign {
    ( $s:expr => ( $( $x:pat => $y:ident ),* $(,)*) ) => {{
        ($(
            if let Some($x) = $s.pop() {
                $y
            } else {
                err_fmt!(
                    "invalid operand in pop assignment: expected {} to be {}",
                    stringify!($y),
                    stringify!($x)
                )?
            }
        ),*)
    }};

    [$s:expr, $r:expr] => {{
        ($r)
            .map(|_| $s.pop().unwrap())
            .collect::<Vec<_>>()
    }};
}

macro_rules! impl_math {
    ($self:expr, $p:path => ($x:path, $y:path) {$f:expr}) => {{
        let (y, x) = pop_assign!($self.stack => (
            $x(x) => x,
            $y(x) => x
        ));
        $self.stack.push($p($f(x, y)));
    }};
}

#[derive(Clone, Debug, PartialEq)]
pub struct Table(pub Vec<(Obj, Obj)>, pub Vec<Obj>);

impl Table {
    #[inline]
    pub fn vec(&self) -> &[Obj] {
        &self.1
    }

    #[inline]
    pub fn vec_mut(&mut self) -> &mut Vec<Obj> {
        &mut self.1
    }

    #[inline]
    pub fn vec_push(&mut self, x: Obj) {
        self.vec_mut().push(x);
    }

    #[inline]
    pub fn vec_pop(&mut self) {
        let _x = self.vec_mut().pop();
        dbgln!("popped {_x:?}");
    }
}

pub trait Machine {
    fn get_table(&self, x: usize) -> &Table;
    fn get_tables(&self) -> &HashMap<usize, Table>;

    fn add_table(&mut self, t: Table) -> usize;
    fn rm_table(&mut self, x: &usize);

    /** push obj x to vec v */
    fn vec_push(&mut self, v: &usize, x: Obj);

    /** push the last item in vec v */
    fn vec_last(&mut self, v: &usize);

    /** pop an item off vec v */
    fn vec_pop(&mut self, v: &usize);
}

#[derive(Debug, Clone, PartialEq)]
pub struct VM<'a> {
    code: &'a [Instr],
    blocks: &'a [Blk],
    bodies: &'a [Body<'a>],
    stack: Vec<Obj>,
    vars: HashMap<usize, Obj>,
    tables: (usize, HashMap<usize, Table>),
}

impl<'a> Machine for VM<'a> {
    fn get_table(&self, x: usize) -> &Table {
        &self.tables.1[&x]
    }

    #[inline]
    fn get_tables(&self) -> &HashMap<usize, Table> {
        &self.tables.1
    }

    fn add_table(&mut self, x: Table) -> usize {
        let n = self.tables.0;
        self.tables.1.insert(n, x);
        self.tables.0 += 1;
        n
    }

    #[inline]
    fn rm_table(&mut self, x: &usize) {
        self.tables.1.remove(x);
    }

    fn vec_push(&mut self, v: &usize, x: Obj) {
        if let Some(Obj::T(t)) = self.vars.get_mut(&v) {
            self.tables.1
                .get_mut(t)
                .expect(&format!(
                    "table {t} not found (referenced by variable {v}"
                ))
                .vec_push(x);
        } else {
            unreachable!()
        }
    }

    fn vec_last(&mut self, v: &usize) {
        if let Obj::T(t) = self.vars[v] {
            let v = self.get_table(t).vec();
            self.stack.push(v[v.len() - 1]);
        } else {
            unreachable!()
        }
    }

    fn vec_pop(&mut self, v: &usize) {
        if let Some(Obj::T(t)) = self.vars.get_mut(&v) {
            self.tables.1
                .get_mut(t)
                .expect(&format!(
                    "table {t} not found (referenced by variable {v}"
                ))
                .vec_pop();
        } else {
            unreachable!()
        }
    }
}

impl<'a> VM<'a> {
    pub fn new(
        code: &'a [Instr],
        blocks: &'a [Blk],
        bodies: &'a [Body<'a>],
    ) -> Self {
        Self {
            code,
            blocks,
            bodies,
            stack: Vec::new(),
            vars: HashMap::new(),
            tables: (0, HashMap::new()),
        }
    }

    pub fn fmt(&self, x: &Obj) -> String {
        use Obj::*;
        match x {
            x @ (C(_) | F(_) | U(_) | Fun(_)) => format!("{x}"),
            T(x) => {
                let t = self.get_table(*x);
                format!(
                    "[{}|{}]",
                    t.0.iter()
                        .map(|(x, y)| format!(
                            "{}: {}",
                            self.fmt(x),
                            self.fmt(y)
                        ))
                        .collect::<Vec<_>>()
                        .join(", "),
                    t.1.iter()
                        .map(|x| self.fmt(x))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }

    fn iter_body(&self, i: usize) -> BodyIterator<'a> {
        BodyIterator::from(&self.code, &self.bodies[i])
    }

    #[cfg(test)]
    fn load_var(&self, i: &usize) -> Res<Obj> {
        self.vars
            .get(&i)
            .map(|x| Ok(*x))
            .unwrap_or(err_fmt!("no var {i}"))
    }

    #[cfg(not(test))]
    #[inline]
    fn load_var(&self, i: &usize) -> Res<Obj> {
        Ok(self.vars[i])
    }

    #[inline]
    fn find_label(&self, x: usize) -> Option<usize> {
        self.code
            .iter()
            .enumerate()
            .find(|(_, i)| i == &&Instr::Label(x))
            .map(|(i, x)| {
                if let Instr::Label(_) = x {
                    i
                } else {
                    unreachable!()
                }
            })
    }

    fn exe_instr(&mut self, x: &Instr) -> Res<()> {
        #[cfg(test)]
        let len = self
            .code
            .iter()
            .map(|x| format!("{x:?}").len())
            .fold(0, |x, y| x.max(y))
            + 2;
        dbgln!(
            "instr: {} {:?}",
            format!("{x:?}")
                .pad_to_width_with_alignment(len, Alignment::Right)
                .purple(),
            self.stack,
        );

        let r = match x {
            Instr::Push(x) => self.stack.push(*x),
            Instr::Pop => {
                let _ = self.stack.pop();
                ()
            }

            Instr::Local(x) => self
                .stack
                .pop()
                .map(|o| self.vars.insert(*x, o))
                .and_then(|_| Some(()))
                .expect("expected variable on stack for local() call"),

            Instr::Load(x) => self.stack.push(self.load_var(x)?),

            Instr::AddF => {
                impl_math!(self, Obj::F => (Obj::F, Obj::F) {|x, y|x+y})
            }
            Instr::SubF => {
                impl_math!(self, Obj::F => (Obj::F, Obj::F) {|x, y|x-y})
            }
            Instr::MulF => {
                impl_math!(self, Obj::F => (Obj::F, Obj::F) {|x, y|x*y})
            }
            Instr::DivF => {
                impl_math!(self, Obj::F => (Obj::F, Obj::F) {|x, y|x/y})
            }

            Instr::CmpF => {
                let (y, x) = pop_assign!(self.stack => (
                        Obj::F(x) => x,
                        Obj::F(x) => x,
                ));
                self.stack.push(Obj::F(if x < y {
                    -1.
                } else if x > y {
                    1.
                } else {
                    0.
                }));
            }

            Instr::NegF => {
                let x = pop_assign!(self.stack => (Obj::F(x) => x));
                self.stack.push(Obj::F(-x));
            }

            Instr::Apply0 => {
                let x = pop_assign!(self.stack => (
                    Obj::Fun(x) => x,
                ));
                let r = self.exe_block(x)?;
                self.stack.push(r);
            }

            Instr::Apply1 => {
                let (y, x) = pop_assign!(self.stack => (
                    y => y,
                    Obj::Fun(x) => x,
                ));

                self.stack.push(y);
                let r = self.exe_block(x)?;
                self.stack.push(r);
            }

            Instr::ApplyN => {
                let f = pop_assign!(self.stack => (Obj::Fun(x) => x));
                let v = pop_assign!(self.stack, 0..self.stack.len());

                v.iter().for_each(|x| self.stack.push(*x));
                let r = self.exe_block(f)?;
                self.stack.push(r);
            }

            Instr::Table(x) => {
                let mut r = Vec::new();
                /* first iter the  arguments and put them in a vec */
                let mut i = pop_assign!(self.stack, 0..2 * *x).into_iter();

                while let Some(a) = i.next() {
                    if let Some(b) = i.next() {
                        r.push((a, b));
                    } else {
                        err_fmt!("non-rectangular stack in Table().")?
                    }
                }

                let t = self.add_table(Table(r, Vec::new()));
                self.stack.push(Obj::T(t));
            }

            Instr::PopVec => {
                let i = pop_assign!(self.stack => (Obj::T(x) => x));
                let v = self
                    .get_table(i)
                    .vec()
                    .iter()
                    .rev()
                    .copied()
                    .collect::<Vec<Obj>>();

                v.into_iter().for_each(|x| self.stack.push(x));
                self.rm_table(&i);
            }

            Instr::Vec(x) => {
                let v = pop_assign!(self.stack, 0..*x)
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>();
                let u = self.add_table(Table(Vec::new(), v));

                self.stack.push(Obj::T(u));
            }

            Instr::VecFull => {
                let v = pop_assign!(self.stack, 0..self.stack.len())
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>();
                let t = self.add_table(Table(Vec::new(), v));

                self.stack.push(Obj::T(t));
            }

            Instr::VecPush(v) => {
                let x = pop_assign!(self.stack => (x => x));
                self.vec_push(v, x);
            }

            Instr::VecLast(v) => {
                self.vec_last(v);
            }

            Instr::Dup => {
                self.stack.push(
                    *self.stack.iter().last().expect("item on stack for dup"),
                );
            }

            Instr::Swap2 => {
                let (y, x) = pop_assign!(self.stack => (
                    x => x,
                    x => x,
                ));

                self.stack.push(y);
                self.stack.push(x);
            }

            Instr::Jmp(i) => {
                let r = self.exe_block(*i)?;
                self.stack.push(r);
            }

            Instr::LJmpNZ(x) => {
                let n = pop_assign!(self.stack => (
                        Obj::F(x) => x,
                ));
                if n != 0. {
                    let i = self.find_label(*x).expect("label not found");
                    dbgln!(
                        alert,
                        "found label {x}. jumping to instruction {i}"
                    );
                    self.exe_at(i)?;
                }
            }

            Instr::Label(_) => (),

            #[allow(unreachable_patterns)]
            x => return err_fmt!("invalid instruction: {x:?}"),
        };

        Ok(r)
    }

    pub fn exe_body(&mut self, i: usize) -> Res<Obj> {
        for x in self.iter_body(i) {
            self.exe_instr(x)?;
        }

        match self.stack.pop() {
            Some(x) => Ok(x),
            None => err_fmt!("invalid return: no value on top of stack"),
        }
    }

    fn exe_at(&mut self, mut i: usize) -> Res<()> {
        while i < self.code.len() {
            let x = self.code[i];
            if x == Instr::Ret {
                break;
            }
            self.exe_instr(&x)?;
            i += 1;
        }
        Ok(())
    }

    pub fn exe_block(&mut self, i: usize) -> Res<Obj> {
        match self.blocks[i] {
            Blk(BlkType::Fun, Time::Immediate, i) => {
                dbgln!(
                    alert,
                    "executing immediate body {i}: jumping to {}",
                    self.bodies[i].start
                );
                self.exe_body(i)
            }
            Blk(BlkType::Fun, Time::Deferred, _) if self.stack.len() == 0 => {
                dbgln!(alert, "deferred block with empty stack. returning.");
                Ok(Obj::Fun(i))
            }
            Blk(BlkType::Fun, Time::Deferred, i)
                if self.bodies[i].vars <= self.stack.len() =>
            {
                dbgln!(
                    alert,
                    "deferred block with correct stack. jumping to {}",
                    self.bodies[i].start
                );
                self.exe_body(i)
            }
            Blk(BlkType::Fun, Time::Deferred, _) => {
                err_fmt!("deferred block called with too few arguments")
            }
            _ => unreachable!(),
        }
        .map_err(|e| {
            heredoc!(
                r#"
                err while executing block {i}: {e}
                tables:
                {t}
                vars:
                {v}
                "#,
                t = self.fmt_tables(),
                v = self.fmt_vars(),
            )
        })
    }

    pub fn fmt_tables(&self) -> String {
        self.tables
            .1
            .iter()
            .map(|(i, Table(t, a))| {
                format!(
                    "{i} /\n{}\n\t--------\n{}",
                    t.iter()
                        .map(|(k, v)| format!(
                            "\t{} :\t{}",
                            self.fmt(k),
                            self.fmt(v)
                        ))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    a.iter()
                        .enumerate()
                        .map(|(i, x)| format!("\t{i:3}: {}", self.fmt(x)))
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn fmt_vars(&self) -> String {
        self.vars
            .iter()
            .map(|(i, x)| format!("{i:3}: '{}'", self.fmt(x)))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn dump_tables(&self) {
        dbgln!("{}", self.fmt_tables());
    }
}

#[cfg(test)]
mod test {
    use crate::{
        lisp::{Leaf, LeafType, Machine},
        or_fatal,
        vm::VM,
    };

    #[test]
    fn expr() {
        use Leaf::*;
        for (i, (x, y)) in [
            (D("+", &F(5.), &F(2.)), "7"),
            (D("*", &F(5.), &F(2.)), "10"),
            (D("-", &F(5.), &F(2.)), "3"),
            (D("%", &F(5.), &F(2.)), "2.5"),
            (
                Fun {
                    a: Vec::new(),
                    v: vec![D("+", &F(1.), &F(1.))],
                },
                "{&1}",
            ),
            (
                D(
                    "@",
                    &Fun {
                        a: vec![("x", LeafType::F)],
                        v: vec![D("+", &F(1.), &X("x"))],
                    },
                    &F(2.),
                ),
                "3",
            ),
            (
                D(
                    ".",
                    &Fun {
                        a: vec![("x", LeafType::F), ("y", LeafType::F)],
                        v: vec![D("+", &X("x"), &X("y"))],
                    },
                    &A(vec![F(1.), F(1.)]),
                ),
                "2",
            ),
            (A(vec![Leaf::F(1.), Leaf::F(2.)]), "[|1, 2]"),
            (M("!", &Leaf::F(3.)), "[|0, 1, 2]"),
            (
                D(
                    ".",
                    &Fun {
                        a: vec![("x", LeafType::F), ("y", LeafType::F)],
                        v: vec![A(vec![X("x"), X("y")])],
                    },
                    &M("!", &F(2.)),
                ),
                "[|0, 1]",
            ),
            (M("!", &D("+", &F(1.), &F(3.))), "[|0, 1, 2, 3]"),
            (M("-", &M("-", &F(1.))), "1"),
        ]
        .into_iter()
        .enumerate()
        {
            println!(" === TEST {i} ===");
            println!("{x:?}");
            let mut m = Machine::new();
            let b = or_fatal(m.compile(&x));
            m.dump();
            println!("beginning execution at block {b}");

            let mut vm = VM::new(&m.instrs, &m.blocks, &m.bodies);
            let e = or_fatal(vm.exe_block(b));

            println!("tables:");
            vm.dump_tables();

            assert_eq!(vm.fmt(&e), y.to_string())
        }
    }
}
