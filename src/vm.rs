use crate::{
    Res,
    bc::{Blk, BlkType, Body, Instr, Obj, Time},
    err_fmt,
};

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

#[derive(Debug, Clone, PartialEq)]
pub struct VM<'a> {
    code: &'a [Instr],
    blocks: &'a [Blk],
    bodies: &'a [Body<'a>],
    stack: Vec<Obj>,
    vars: Vec<(u8, Obj)>,
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
            vars: Vec::new(),
        }
    }

    fn exe_instr(&mut self, x: &Instr) -> Res<()> {
        println!("instr:\t{:?}", x);
        println!("stack:\t{:?}", self.stack);

        let r = match x {
            Instr::Push(x) => self.stack.push(*x),
            Instr::Pop => self.stack.pop().and_then(|_| Some(())).unwrap_or(()),
            Instr::Local(x) => self
                .stack
                .pop()
                .and_then(|o| Some(self.vars.push((*x, o))))
                .expect("expected variable on stack for local() call"),
            Instr::Load(x) => self.stack.push(*self
                .vars
                .iter()
                .filter(|(i, _)| i == x)
                .map(|(_, o)| o)
                .last()
                .expect("expected variable {x} in var vector")),


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
                println!("stack in apply1: {:?}", self.stack);
                let r = self.exe_block(x)?;
                self.stack.push(r);
            }

            Instr::Swap2 => {
                let (y, x) = pop_assign!(self.stack => (
                    x => x,
                    x => x,
                ));

                self.stack.push(y);
                self.stack.push(x);
            }

            #[allow(unreachable_patterns)]
            x => return err_fmt!("invalid instruction: {x:?}"),
        };

        Ok(r)
    }

    pub fn exe_body(&mut self, i: usize) -> Res<Obj> {
        let b = &self.bodies[i];
        let mut src = self.code.iter().skip(b.start);

        while let Some(x) = src.next() {
            if x == &Instr::Ret {
                break;
            }
            self.exe_instr(x)?;
        }

        match self.stack.pop() {
            Some(x) => Ok(x),
            None => err_fmt!("invalid return: no value on top of stack"),
        }
    }

    pub fn exe_block(&mut self, i: usize) -> Res<Obj> {
        println!("{:?}", self.blocks[i]);
        match self.blocks[i] {
            Blk(BlkType::Fun, Time::Immediate, i) => {
                println!(
                    "executing immediate body {i}: jumping to {}",
                    self.bodies[i].start
                );
                self.exe_body(i)
            }
            Blk(BlkType::Fun, Time::Deferred, _)
                if self.stack.len() == 0 =>
            {
                println!("deferred block with empty stack. returning.");
                Ok(Obj::Fun(i))
            }
            Blk(BlkType::Fun, Time::Deferred, i)
                if self.bodies[i].vars <= self.stack.len() =>
            {
                println!(
                    "deferred block with correct stack. jumping to {}",
                    self.bodies[i].start
                );
                self.exe_body(i)
            }
            Blk(BlkType::Fun, Time::Deferred, i) => {
                err_fmt!("deferred block called with too few arguments")
            }
            _ => todo!(),
        }
    }

    pub fn exe_last_block(&mut self) -> Res<Obj> {
        println!("blocks: {}", self.blocks.len());
        match self.blocks[self.blocks.len() - 1] {
            Blk(BlkType::Fun, Time::Immediate, i) => {
                println!("executing body {i}");
                self.exe_body(i)
            }
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        lisp::{Leaf, LeafType, Machine},
        vm::VM,
    };

    #[test]
    #[should_panic]
    fn panic_expr() {
        use Leaf::*;
        for x in [D("+", &X("x"), &F(1.))].into_iter() {
            let mut m = Machine::new();
            m.compile(&x).unwrap();
            let mut vm = VM::new(&m.instrs, &m.blocks, &m.bodies);
            let e = vm.exe_last_block().unwrap();
        }
    }

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
        ]
        .into_iter()
        .enumerate()
        {
            println!(" === TEST {i} ===");
            let mut m = Machine::new();
            let b = m.compile(&x).unwrap();

            println!("instrs:");
            m.instrs
                .iter()
                .enumerate()
                .for_each(|(i, x)| println!("{i:3}: {x:?}"));
            println!("bodies:");
            m.bodies
                .iter()
                .enumerate()
                .for_each(|(i, x)| println!("{i:3}: {x:?}"));
            println!("blocks:");
            m.blocks
                .iter()
                .enumerate()
                .for_each(|(i, x)| println!("{i:3}: {x:?}"));
            println!("beginning execution at block {b}");

            let mut vm = VM::new(&m.instrs, &m.blocks, &m.bodies);
            let e = vm.exe_block(b).unwrap();
            assert_eq!(format!("{e}"), y.to_string())
        }
    }
}
