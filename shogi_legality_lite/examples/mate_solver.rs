use shogi_core::{PartialPosition, ToUsi};
use shogi_usi_parser::FromUsi;
use std::env::args;

use shogi_legality_lite::mate_solver::{solve_mate_problem, MateResult};

fn main() {
    let args: Vec<_> = args().collect();
    let mut sfen = "sfen ".to_string();
    if args.len() >= 2 {
        sfen += &args[1];
    } else {
        std::io::stdin().read_line(&mut sfen).unwrap();
    }
    let position = PartialPosition::from_usi(sfen.trim()).unwrap();
    let max_depth = 15;
    let mut depth = 1;
    let mut result = MateResult::default();
    while depth <= max_depth {
        result = solve_mate_problem(&position, depth);
        if result.is_mate {
            break;
        }
        depth += 2;
    }
    if result.is_mate {
        println!("Mate found:");
        println!("#nodes visited: {}", result.nodes);
        println!("Length: {}", result.pv_rev.len());
        println!("Moves:");
        result.pv_rev.reverse();
        for (index, mv) in result.pv_rev.into_iter().enumerate() {
            println!("{}: {}", index + 1, mv.to_usi_owned());
        }
    } else {
        println!("Mate not found");
        println!("#nodes visited: {}", result.nodes);
    }
}
