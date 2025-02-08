use shogi_core::PartialPosition;

pub struct Stat {
    pub all: u64,
}

pub fn perft(pos: PartialPosition, depth: usize) -> Stat {
    if depth == 0 {
        return Stat { all: 1 };
    }
    let all = crate::all_legal_moves_partial(&pos);
    if depth == 1 {
        return Stat {
            all: all.len() as u64,
        };
    }
    let mut answer_all = 0;
    for mv in all {
        let mut next = pos.clone();
        next.make_move(mv).unwrap();
        let sub = perft(next, depth - 1);
        answer_all += sub.all;
    }
    Stat { all: answer_all }
}

#[cfg(test)]
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
        for (depth, &expected) in TABLE_ALL[..4].iter().enumerate() {
            let pos = PartialPosition::startpos();
            let result = perft(pos, depth);
            assert_eq!(result.all, expected);
        }
    }
}
