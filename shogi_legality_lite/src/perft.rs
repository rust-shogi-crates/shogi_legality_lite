use shogi_core::{LegalityChecker, PartialPosition};

use crate::LiteLegalityChecker;

struct Stat {
    all: u64,
}

fn perft(pos: PartialPosition, depth: usize) -> Stat {
    if depth == 0 {
        return Stat { all: 1 };
    }
    let all = LiteLegalityChecker.all_legal_moves_partial(&pos);
    let mut answer_all = 0;
    for mv in all {
        let mut next = pos.clone();
        next.make_move(mv).unwrap();
        let sub = perft(next, depth - 1);
        answer_all += sub.all;
    }
    Stat { all: answer_all }
}

mod tests {
    use super::*;
    // Table is retrieved from https://qiita.com/ak11/items/8bd5f2bb0f5b014143c8.
    const TABLE_ALL: [u64; 9] = [
        1,
        30,
        900,
        25470,
        719731,
        19861490,
        547581517,
        15086269607,
        416062133009,
    ];

    #[test]
    fn perft_result_matches() {
        for (depth, &expected) in TABLE_ALL[..3].iter().enumerate() {
            let pos = PartialPosition::startpos();
            let result = perft(pos, depth);
            assert_eq!(result.all, expected);
        }
    }

    #[cfg(bench)]
    #[bench]
    fn bench_perft(b: &mut test::Bencher) {
        let depth = 2;
        let expected = TABLE_ALL[depth];
        b.iter(|| {
            let pos = PartialPosition::startpos();
            let result = perft(pos, depth);
            assert_eq!(result.all, expected);
        });
    }
}