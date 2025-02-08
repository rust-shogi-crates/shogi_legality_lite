# https://bheisler.github.io/criterion.rs/book/faq.html#cargo-bench-gives-unrecognized-option-errors-for-valid-command-line-options
cargo bench --all --bench perft -- --output-format bencher
cargo bench --all --bench mate_solver -- --output-format bencher
