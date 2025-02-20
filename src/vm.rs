use crate::{
    Res,
    bc::{Block, Body, Instr, Obj},
    err_fmt,
};

macro_rules! pop_assign {
    ( $s:expr => ( $( $x:pat => $y:ident ),* ) ) => {{
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

#[derive(Debug, Clone, PartialEq)]
pub struct VM<'a> {
    code: &'a [Instr],
    blocks: &'a [Block],
    bodies: &'a [Body<'a>],
    stack: Vec<Obj>,
}

impl<'a> VM<'a> {
    pub fn new(
        code: &'a [Instr],
        blocks: &'a [Block],
        bodies: &'a [Body<'a>],
    ) -> Self {
        Self {
            code,
            blocks,
            bodies,
            stack: Vec::new(),
        }
    }

    /*
    pub fn exe_block(&mut self, i: usize) -> Res<Obj> {
        Ok(match self.blocks[i] {
            Block(Fun, Immediate, i) => {
            }
            x => return err_fmt!("invalid block {x:?}"),
        })
    }
    */

    fn exe_instr(&mut self, x: &Instr) -> Res<()> {
        Ok(match x {
            Instr::Push(x) => self.stack.push(*x),
            Instr::Pop => self.stack.pop().and_then(|_| Some(())).unwrap_or(()),

            Instr::AddI => {
                let y = if let Some(Obj::I(y)) = self.stack.pop() {
                    y
                } else {
                    err_fmt!("invalid operand in integer addition: expected y to be int")?
                };
                let x = if let Some(Obj::I(x)) = self.stack.pop() {
                    x
                } else {
                    err_fmt!("invalid operand in integer addition: expected x be int")?
                };
                self.stack.push(Obj::I(x + y));
            }

            Instr::SubI => {
                let (y, x) = pop_assign!(self.stack => (
                    Obj::I(x) => x,
                    Obj::I(x) => x
                ));
                self.stack.push(Obj::I(x - y));
            }

            Instr::MulI => {
                let (y, x) = pop_assign!(self.stack => (
                    Obj::I(x) => x,
                    Obj::I(x) => x
                ));
                self.stack.push(Obj::I(x * y));
            }

            Instr::DivI => {
                let (y, x) = pop_assign!(self.stack => (
                    Obj::I(x) => x,
                    Obj::I(x) => x
                ));
                self.stack.push(Obj::I(x / y));
            }

            #[allow(unreachable_patterns)]
            x => return err_fmt!("invalid instruction: {x:?}"),
        })
    }

    pub fn exe_body(&mut self, i: usize) -> Res<Obj> {
        let b = &self.bodies[i];
        let mut src = self.code.iter().skip(b.start);

        while let Some(x) = src.next() {
            if x == &Instr::Ret { break; }
            self.exe_instr(x)?;
        }

        match self.stack.pop() {
            Some(x) => Ok(x),
            None => err_fmt!("invalid return: no value on top of stack"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        lisp::{Leaf, Machine},
        vm::VM,
    };

    #[test]
    fn expr() {
        for (x, y) in [
            (Leaf::I(1), "1"),
            (Leaf::D("+", &Leaf::I(1), &Leaf::I(2)), "3"),
            (Leaf::D("-", &Leaf::I(5), &Leaf::I(2)), "3"),
        ]
            .into_iter()
        {
            let mut m = Machine::new();
            m.compile(&x).unwrap();
            let mut vm = VM::new(&m.instrs, &m.blocks, &m.bodies);
            let e = vm.exe_body(0).unwrap();
            assert_eq!(format!("{e}"), y.to_string())
        }
    }
}
