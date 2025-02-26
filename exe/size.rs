use nvm::bc::{Blk, Body, Instr, Obj, ObjType};

macro_rules! sizeof_each {
    [$($t:ty),*] => {{
        [ $( (stringify!($t), std::mem::size_of::<$t>()) ),* ]
    }};
}

fn main() {
    sizeof_each![Instr, Body, Blk, Obj, ObjType]
        .into_iter()
        .for_each(|(x, size)| println!("{x:7} {:4}", size * 8));
}
