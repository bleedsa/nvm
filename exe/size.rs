use nvm::bc::{Block, Body, Instr, Obj};

macro_rules! sizeof_each {
    [$($t:ty),*] => {{
        [ $( (stringify!($t), std::mem::size_of::<$t>()) ),* ]
    }};
}

fn main() {
    sizeof_each![Instr, Body, Block, Obj]
        .into_iter()
        .for_each(|(x, size)| println!("{x}:\t{}", size * 8));
}
