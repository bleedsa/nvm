use crate::{Res, err_fmt, bc::{Instr, Obj, Block, Body}};

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

    pub fn exe_body(&mut self, i: usize) -> Res<Obj> {
        let mut s: Vec<Obj> = Vec::new();
        let b = &self.bodies[i];
        let mut src = self.code.iter().skip(b.start);

        while let Some(x) = src.next() {
            match x {
                Instr::Push(x) => s.push(*x),
                Instr::Ret => break,

                #[allow(unreachable_patterns)]
                x => return err_fmt!("invalid instruction in block: {x:?}"),
            }
        }

        match s.pop() {
            Some(x) => Ok(x),
            None => err_fmt!("invalid return: no value on top of stack"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{lisp::{Leaf, Machine}, vm::VM};

    #[test]
    fn expr() {
        for (x, y) in [
            (Leaf::I(1), "1"),
        ].into_iter() {
            let mut m = Machine::new();
            m.compile(&x).unwrap();
            let mut vm = VM::new(&m.instrs, &m.blocks, &m.bodies);
            let e = vm.exe_body(0).unwrap();
            assert_eq!(format!("{e}"), y.to_string())
        }
    }
}
