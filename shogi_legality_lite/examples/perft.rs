use std::env::args;

use shogi_core::PartialPosition;

fn main() {
    let args: Vec<_> = args().collect();
    let mut depth = 5;
    if args.len() >= 2 {
        depth = args[1].parse().unwrap();
    }
    let pos = PartialPosition::startpos();
    let result = shogi_legality_lite::perft::perft(pos, depth);
    println!("{} => {}", depth, result.all);
}
