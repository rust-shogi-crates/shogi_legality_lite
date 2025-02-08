use criterion::{criterion_group, criterion_main, Criterion};
use shogi_core::{PartialPosition, ToUsi};
use shogi_legality_lite::mate_solver::solve_mate_problem;

fn bench_mate_0(c: &mut Criterion) {
    use shogi_usi_parser::FromUsi;

    let mut group = c.benchmark_group("mate");
    group.sample_size(10);
    // From https://github.com/koba-e964/shogi-mate-problems/blob/d58d61336dd82096856bc3ac0ba372e5cd722bc8/2022-05-18/mate5.psn#L3
    group.bench_function("mate 0", |b| {
        let position =
            PartialPosition::from_usi("sfen 3g1ks2/6g2/4S4/7B1/9/9/9/9/9 b G2rbg2s4n4l18p 1")
                .unwrap();
        b.iter(|| {
            let (result, _stat) = solve_mate_problem(&position, 5);
            assert!(result.is_mate);
        })
    });
}

fn bench_mate_1(c: &mut Criterion) {
    use shogi_usi_parser::FromUsi;

    let mut group = c.benchmark_group("mate");
    group.sample_size(10);
    // From https://github.com/koba-e964/shogi-mate-problems/blob/d58d61336dd82096856bc3ac0ba372e5cd722bc8/2022-05-18/mate9.psn#L3
    let position =
        PartialPosition::from_usi("sfen 5kgnl/9/4+B1pp1/8p/9/9/9/9/9 b 2S2rb3g2s3n3l15p 1")
            .unwrap();
    group.bench_function("mate 1", |b| {
        b.iter(|| {
            let (result, _stat) = solve_mate_problem(&position, 9);
            assert!(result.is_mate);
            let mut pv = result.pv_rev;
            pv.reverse();
            assert_eq!(pv[0].to_usi_owned(), "S*5b");
            assert_eq!(pv[1].to_usi_owned(), "4a3b");
        })
    });
}

criterion_group!(benches, bench_mate_0, bench_mate_1);
criterion_main!(benches);
