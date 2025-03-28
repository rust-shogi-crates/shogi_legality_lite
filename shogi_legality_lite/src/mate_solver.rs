use shogi_core::{Move, PartialPosition};

use crate::prelegality::is_mate;

#[derive(Debug, Clone, Default)]
pub struct MateResult {
    pub is_mate: bool,
    pub pv_rev: alloc::vec::Vec<Move>,
}

#[derive(Debug, Clone, Default)]
pub struct SearchStats {
    pub nodes: u64,
    pub edges: u64,
}

pub fn solve_mate_problem(position: &PartialPosition, depth: usize) -> (MateResult, SearchStats) {
    all_mymoves(position, depth)
}

fn all_mymoves(position: &PartialPosition, depth: usize) -> (MateResult, SearchStats) {
    if depth == 0 {
        return (
            MateResult {
                is_mate: false,
                pv_rev: alloc::vec::Vec::new(),
            },
            SearchStats { nodes: 1, edges: 0 },
        );
    }
    let mut all = crate::all_checks_partial(position);
    order_mymoves(position, &mut all);
    let mut nodes = 0;
    let mut edges = 0;
    let mut fastest = alloc::vec::Vec::new();
    for mv in all {
        edges += 1;
        let mut next = position.clone();
        next.make_move(mv).unwrap();
        let (sub, substat) = all_countermoves(
            &next,
            if fastest.is_empty() {
                depth - 1
            } else {
                fastest.len() - 3
            },
        );
        nodes += substat.nodes;
        edges += substat.edges;
        let mut pv_rev = sub.pv_rev;
        pv_rev.push(mv);
        if sub.is_mate {
            if pv_rev.len() == 1 {
                return (
                    MateResult {
                        is_mate: true,
                        pv_rev,
                    },
                    SearchStats { nodes, edges },
                );
            }
            let old_len = if fastest.is_empty() {
                usize::MAX
            } else {
                fastest.len()
            };
            if old_len > pv_rev.len() {
                fastest = pv_rev;
            }
        }
    }
    (
        MateResult {
            is_mate: !fastest.is_empty(),
            pv_rev: fastest,
        },
        SearchStats { nodes, edges },
    )
}

fn all_countermoves(position: &PartialPosition, depth: usize) -> (MateResult, SearchStats) {
    if depth == 0 {
        return (
            MateResult {
                is_mate: matches!(is_mate(position), Some(true)),
                pv_rev: alloc::vec::Vec::new(),
            },
            SearchStats { nodes: 1, edges: 0 },
        );
    }
    let mut all = crate::all_legal_moves_partial(position);
    let mut nodes = 0;
    let mut edges = 0;
    let mut best = alloc::vec::Vec::new();
    order_countermoves(position, &mut all);
    for mv in all {
        edges += 1;
        let mut next = position.clone();
        next.make_move(mv).unwrap();
        let (sub, substat) = all_mymoves(&next, depth - 1);
        nodes += substat.nodes;
        edges += substat.edges;
        let mut pv_rev = sub.pv_rev;
        pv_rev.push(mv);
        if !sub.is_mate {
            return (
                MateResult {
                    is_mate: false,
                    pv_rev,
                },
                SearchStats { nodes, edges },
            );
        }
        if best.len() < pv_rev.len() {
            best = pv_rev;
        }
    }
    (
        MateResult {
            is_mate: true,
            pv_rev: best,
        },
        SearchStats { nodes, edges },
    )
}

fn order_mymoves(position: &PartialPosition, moves: &mut [Move]) {
    let side = position.side_to_move();
    moves.sort_unstable_by_key(|&mv| {
        let mut score = 0;
        match mv {
            Move::Normal { from, to, promote } => {
                if !promote && (from.relative_rank(side) <= 3 || to.relative_rank(side) <= 3) {
                    score = 20;
                }
            }
            Move::Drop { .. } => score = -10,
        }
        score
    });
}

fn order_countermoves(position: &PartialPosition, moves: &mut [Move]) {
    moves.sort_unstable_by_key(|&mv| {
        let mut score = 0;
        // is mv a capture?
        if let Move::Normal {
            from: _,
            to,
            promote,
        } = mv
        {
            if position.piece_at(to).is_some() {
                score = -10;
            }
            if !promote {
                score += 1;
            }
        }
        score
    });
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
        let (result, _stat) = solve_mate_problem(&position, 5);
        assert!(result.is_mate);
    }

    #[test]
    fn solve_mate_problem_works_1() {
        use shogi_usi_parser::FromUsi;

        // From https://github.com/koba-e964/shogi-mate-problems/blob/d58d61336dd82096856bc3ac0ba372e5cd722bc8/2022-05-18/mate9.psn#L3
        let position =
            PartialPosition::from_usi("sfen 5kgnl/9/4+B1pp1/8p/9/9/9/9/9 b 2S2rb3g2s3n3l15p 1")
                .unwrap();
        let (result, _stat) = solve_mate_problem(&position, 9);
        assert!(result.is_mate);
        let mut pv = result.pv_rev;
        pv.reverse();
        assert_eq!(pv[0].to_usi_owned(), "S*5b");
        assert_eq!(pv[1].to_usi_owned(), "4a3b");
    }
}
