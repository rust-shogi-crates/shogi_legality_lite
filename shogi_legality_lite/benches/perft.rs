use criterion::{criterion_group, criterion_main, Criterion};
use shogi_core::PartialPosition;
use shogi_legality_lite::perft::perft;

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

fn bench_perft(c: &mut Criterion) {
    let mut group = c.benchmark_group("perft");
    group.sample_size(10);
    #[allow(clippy::needless_range_loop)]
    for depth in 0..=4 {
        group.bench_function(format!("perft {depth}"), |b| {
            let expected = TABLE_ALL[depth];
            b.iter(|| {
                let pos = PartialPosition::startpos();
                let result = perft(pos, depth);
                assert_eq!(result.all, expected);
            })
        });
    }
}

criterion_group!(benches, bench_perft);
criterion_main!(benches);
