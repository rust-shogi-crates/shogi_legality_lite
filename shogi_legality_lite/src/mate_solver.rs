use shogi_core::{LegalityChecker, Move, PartialPosition};

use crate::{prelegality::is_mate, LiteLegalityChecker};

#[derive(Debug, Clone, Default)]
pub struct MateResult {
    pub is_mate: bool,
    pub nodes: u64,
    pub pv_rev: alloc::vec::Vec<Move>,
}

pub fn solve_mate_problem(position: &PartialPosition, depth: usize) -> MateResult {
    all_mymoves(position, depth)
}

fn all_mymoves(position: &PartialPosition, depth: usize) -> MateResult {
    if depth == 0 {
        return MateResult {
            is_mate: false,
            nodes: 1,
            pv_rev: alloc::vec::Vec::new(),
        };
    }
    let all = crate::all_checks_partial(position);
    let mut nodes = 0;
    let mut fastest = alloc::vec::Vec::new();
    for mv in all {
        let mut next = position.clone();
        next.make_move(mv).unwrap();
        let sub = all_countermoves(&next, depth - 1);
        nodes += sub.nodes;
        let mut pv_rev = sub.pv_rev;
        pv_rev.push(mv);
        if sub.is_mate {
            if pv_rev.len() == 1 {
                return MateResult {
                    is_mate: true,
                    nodes,
                    pv_rev,
                };
            }
            let old_len = if fastest.is_empty() {
                usize::max_value()
            } else {
                fastest.len()
            };
            if old_len > pv_rev.len() {
                fastest = pv_rev;
            }
        }
    }
    MateResult {
        is_mate: !fastest.is_empty(),
        nodes,
        pv_rev: fastest,
    }
}

fn all_countermoves(position: &PartialPosition, depth: usize) -> MateResult {
    if depth == 0 {
        return MateResult {
            is_mate: matches!(is_mate(position), Some(true)),
            nodes: 1,
            pv_rev: alloc::vec::Vec::new(),
        };
    }
    let all = LiteLegalityChecker.all_legal_moves_partial(position);
    let mut nodes = 0;
    let mut best = alloc::vec::Vec::new();
    for mv in all {
        let mut next = position.clone();
        next.make_move(mv).unwrap();
        let sub = all_mymoves(&next, depth - 1);
        nodes += sub.nodes;
        let mut pv_rev = sub.pv_rev;
        pv_rev.push(mv);
        if !sub.is_mate {
            return MateResult {
                is_mate: false,
                nodes,
                pv_rev,
            };
        }
        if best.len() < pv_rev.len() {
            best = pv_rev;
        }
    }
    MateResult {
        is_mate: true,
        nodes,
        pv_rev: best,
    }
}

#[cfg(test)]
mod tests {
    use shogi_core::ToUsi;

    use super::*;

    #[test]
    fn solve_mate_problem_works_0() {
        use shogi_usi_parser::FromUsi;

        // From https://github.com/koba-e964/shogi-mate-problems/blob/d58d61336dd82096856bc3ac0ba372e5cd722bc8/2022-05-18/mate5.psn#L3
        let position =
            PartialPosition::from_usi("sfen 3g1ks2/6g2/4S4/7B1/9/9/9/9/9 b G2rbg2s4n4l18p 1")
                .unwrap();
        let result = solve_mate_problem(&position, 5);
        assert!(result.is_mate);
    }

    #[test]
    fn solve_mate_problem_works_1() {
        use shogi_usi_parser::FromUsi;

        // From https://github.com/koba-e964/shogi-mate-problems/blob/d58d61336dd82096856bc3ac0ba372e5cd722bc8/2022-05-18/mate9.psn#L3
        let position =
            PartialPosition::from_usi("sfen 5kgnl/9/4+B1pp1/8p/9/9/9/9/9 b 2S2rb3g2s3n3l15p 1")
                .unwrap();
        let result = solve_mate_problem(&position, 9);
        assert!(result.is_mate);
        let mut pv = result.pv_rev;
        pv.reverse();
        assert_eq!(pv[0].to_usi_owned(), "S*5b");
        assert_eq!(pv[1].to_usi_owned(), "4a3b");
    }
}
