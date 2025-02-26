use criterion::{Criterion, criterion_group, criterion_main};
use nvm::{
    lisp::{Leaf, LeafType, Machine},
    vm::VM,
};

fn bench(c: &mut Criterion) {
    use Leaf::*;
    for (i, x) in [
        D("+", &F(1.), &F(1.)),
        D(
            ".",
            &Fun {
                a: vec![("x", LeafType::F), ("y", LeafType::F)],
                v: vec![A(vec![X("x"), X("y")])],
            },
            &M("!", &F(2.)),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let mut m = Machine::new();
        let b = m.compile(&x).unwrap();

        let mut vm = VM::new(&m.instrs, &m.blocks, &m.bodies);
        c.bench_function(&format!("bc {i}"), |ctx| {
            ctx.iter(|| vm.exe_block(b).unwrap())
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
