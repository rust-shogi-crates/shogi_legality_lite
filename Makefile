CALLGRAPH_JSON ?= callgraph.json
CALLGRAPH_RAW_JSON ?= target/callgraph/callgraph.raw.json
CALLGRAPH_CHECK_JSON ?= target/callgraph/callgraph.check.json
CALLGRAPH_SVG ?= callgraph.svg
CALLGRAPH_CHECK_SVG ?= target/callgraph/callgraph.check.svg
CALLGRAPH_CYCLES_JSON ?= callgraph_cycles.json
CALLGRAPH_CHECK_CYCLES_JSON ?= target/callgraph/callgraph_cycles.check.json
CARGO_CALLGRAPH ?= cargo callgraph
PYTHON ?= python3

all: sharedlib include c_tests

.PHONY: sharedlib include c_tests callgraph-json check-callgraph-json callgraph-svg check-callgraph-svg callgraph-cycles check-callgraph-cycles

sharedlib:
	cargo +nightly build --release --no-default-features --features alloc

include: include/shogi_legaity_lite.h

include/shogi_legaity_lite.h: cbindgen.toml
	./generate_header.sh

check-include:
	./generate_header.sh --verify

c_tests: sharedlib include
	$(MAKE) -C c_tests

callgraph-json:
	mkdir -p $(dir $(CALLGRAPH_RAW_JSON)) $(dir $(CALLGRAPH_JSON))
	$(CARGO_CALLGRAPH) --manifest-path Cargo.toml --json $(CALLGRAPH_RAW_JSON)
	$(PYTHON) scripts/normalize_callgraph_json.py $(CALLGRAPH_RAW_JSON) $(CALLGRAPH_JSON)

check-callgraph-json:
	mkdir -p $(dir $(CALLGRAPH_RAW_JSON)) $(dir $(CALLGRAPH_CHECK_JSON))
	$(CARGO_CALLGRAPH) --manifest-path Cargo.toml --json $(CALLGRAPH_RAW_JSON)
	$(PYTHON) scripts/normalize_callgraph_json.py $(CALLGRAPH_RAW_JSON) $(CALLGRAPH_CHECK_JSON)
	cmp $(CALLGRAPH_JSON) $(CALLGRAPH_CHECK_JSON)

callgraph-svg:
	mkdir -p $(dir $(CALLGRAPH_SVG))
	$(PYTHON) scripts/callgraph_json_to_svg.py $(CALLGRAPH_JSON) $(CALLGRAPH_SVG)

check-callgraph-svg:
	mkdir -p $(dir $(CALLGRAPH_CHECK_SVG))
	$(PYTHON) scripts/callgraph_json_to_svg.py $(CALLGRAPH_JSON) $(CALLGRAPH_CHECK_SVG)
	cmp $(CALLGRAPH_SVG) $(CALLGRAPH_CHECK_SVG)

callgraph-cycles:
	mkdir -p $(dir $(CALLGRAPH_CYCLES_JSON))
	$(PYTHON) scripts/callgraph_cycles.py $(CALLGRAPH_JSON) $(CALLGRAPH_CYCLES_JSON)

check-callgraph-cycles:
	mkdir -p $(dir $(CALLGRAPH_CHECK_CYCLES_JSON))
	$(PYTHON) scripts/callgraph_cycles.py $(CALLGRAPH_JSON) $(CALLGRAPH_CHECK_CYCLES_JSON)
	cmp $(CALLGRAPH_CYCLES_JSON) $(CALLGRAPH_CHECK_CYCLES_JSON)
