all: sharedlib include c_tests

.PHONY: sharedlib include c_tests

sharedlib:
	cargo +nightly build --release --no-default-features --features alloc

include: include/shogi_legaity_lite.h

include/shogi_legaity_lite.h: cbindgen.toml
	./generate_header.sh

check-include:
	./generate_header.sh --verify

c_tests: sharedlib include
	$(MAKE) -C c_tests
