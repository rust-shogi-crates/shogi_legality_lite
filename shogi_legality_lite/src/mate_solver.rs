use shogi_core::{LegalityChecker, Move, PartialPosition};

use crate::{prelegality::is_mate, LiteLegalityChecker};

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
    let all = crate::all_checks_partial(position);
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
                usize::max_value()
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
    let all = LiteLegalityChecker.all_legal_moves_partial(position);
    let mut nodes = 0;
    let mut edges = 0;
    let mut best = alloc::vec::Vec::new();
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

    #[cfg(bench)]
    #[bench]
    fn bench_mate_0(b: &mut test::Bencher) {
        use shogi_usi_parser::FromUsi;

        // From https://github.com/koba-e964/shogi-mate-problems/blob/d58d61336dd82096856bc3ac0ba372e5cd722bc8/2022-05-18/mate5.psn#L3
        let position =
            PartialPosition::from_usi("sfen 3g1ks2/6g2/4S4/7B1/9/9/9/9/9 b G2rbg2s4n4l18p 1")
                .unwrap();
        b.iter(|| {
            let (result, _stat) = solve_mate_problem(&position, 5);
            assert!(result.is_mate);
        });
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

    #[cfg(bench)]
    #[bench]
    fn bench_mate_1(b: &mut test::Bencher) {
        use shogi_usi_parser::FromUsi;

        // From https://github.com/koba-e964/shogi-mate-problems/blob/d58d61336dd82096856bc3ac0ba372e5cd722bc8/2022-05-18/mate9.psn#L3
        let position =
            PartialPosition::from_usi("sfen 5kgnl/9/4+B1pp1/8p/9/9/9/9/9 b 2S2rb3g2s3n3l15p 1")
                .unwrap();
        b.iter(|| {
            let (result, _stat) = solve_mate_problem(&position, 9);
            assert!(result.is_mate);
            let mut pv = result.pv_rev;
            pv.reverse();
            assert_eq!(pv[0].to_usi_owned(), "S*5b");
            assert_eq!(pv[1].to_usi_owned(), "4a3b");
        })
    }
}
