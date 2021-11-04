pub use core::*;
pub use criterion::{
    Criterion,
    criterion_group, 
    criterion_main, 
    black_box, 
};
/*
pub use pretty_assertions::{
    assert_eq,
    assert_ne,
};
*/

mod fen;
mod perft;


criterion_main!(perft::benches);
//criterion_main!(fen::benches);
