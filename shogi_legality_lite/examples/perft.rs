use shogi_core::PartialPosition;

fn main() {
    let depth = 5;
    let pos = PartialPosition::startpos();
    let result = shogi_legality_lite::perft::perft(pos, depth);
    println!("{} => {}", depth, result.all);
}
