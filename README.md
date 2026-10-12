# Rust shogi crates: Legality Checker (lite version)
[![Rust](https://github.com/rust-shogi-crates/shogi_legality_lite/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/rust-shogi-crates/shogi_legality_lite/actions/workflows/rust.yml?query=branch%3Amain)
[![C bindings](https://github.com/rust-shogi-crates/shogi_legality_lite/actions/workflows/c-bindings.yml/badge.svg?branch=main)](https://github.com/rust-shogi-crates/shogi_legality_lite/actions/workflows/c-bindings.yml?query=branch%3Amain)
![Rust Version](https://img.shields.io/badge/rustc-1.60+-blue.svg)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/mit-license.php)

This repository handles legality checking of moves in shogi. It consists of two crates: a library crate that defines items (`rlib` crate), and a library crate that defines C bindings to them (`cdylib` crate).

Functions in the `rlib` crate use no constant tables. Crates in this repository are `no_std`-aware, which means they are useful in embedded systems as well as ordinary applications.

Benchmark results are available at: <https://rust-shogi-crates.github.io/shogi_legality_lite/dev/bench/>

## Callgraph JSON

Install [`cargo-callgraph`](https://crates.io/crates/cargo-callgraph), then generate a JSON callgraph:

```sh
cargo install cargo-callgraph --version 0.1.0 --locked
make callgraph-json
```

The default output path is the checked-in `callgraph.json`. Override it with `CALLGRAPH_JSON=...` if needed.
Use `make check-callgraph-json` to verify that the checked-in file matches a fresh generation.

Generate the checked-in SVG image from `callgraph.json` with:

```sh
make callgraph-svg
```

The SVG shows crate-internal legality nodes, omits `mate_solver` and `perft` modules, and groups nodes by source file.
Use `make check-callgraph-svg` to verify that the checked-in `callgraph.svg` matches `callgraph.json`.
