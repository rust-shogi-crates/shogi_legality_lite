# Must be run in the crate's root directory
cbindgen --config cbindgen.toml --parse-dependencies --crate shogi_legality_lite --output include/shogi_legality_lite.h $@
