# Rust shogi crates: Legality Checker (lite version) (`rlib`)
[![crate](https://img.shields.io/crates/v/shogi_legality_lite)](https://crates.io/crates/shogi_legality_lite)
[![docs](https://docs.rs/shogi_legality_lite/badge.svg)](https://docs.rs/shogi_legality_lite)
![Rust Version](https://img.shields.io/badge/rustc-1.60+-blue.svg)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/mit-license.php)

This crate handles legality checking of moves in shogi.

Functions in this crate use no constant tables. This crate is `no_std`-aware, which means this crate is useful in embedded systems as well as ordinary applications.

Benchmark results are available at <https://rust-shogi-crates.github.io/shogi_legality_lite/dev/bench/>.

## Available features
- `alloc`: `alloc`-related functionalities are made available. Enabled by default.
- `std`: `std`-related functionalities are made available. Implies `alloc`. Enabled by default.
