# The NX remote desktop preload aborts rustfmt/clippy on this host.
unexport LD_PRELOAD

GUIDE ?= pd0-pd1
SOURCES ?= sources/PD0.pdf sources/PD1.pdf
SOURCE ?= sources/PD0.pdf
PAGES ?= 1
DATA ?= guides/$(GUIDE)/data
OUTPUT ?= output/PD0-PD1_Calisma_Ozeti.pdf
VISUAL_BASELINE ?= tests/baselines/pd0-pd1.pdf
VISUAL_PAGES ?= 1,4,8,12,18,22

.PHONY: help build check verify audit doctor parity parity-capture visual visual-capture test fmt clippy analyze pages scaffold

help:
	@echo "make build      # guide PDF"
	@echo "make check      # build + coverage gate"
	@echo "make verify     # check + layout audit + pdfinfo"
	@echo "make audit      # layout audit of the built PDF"
	@echo "make doctor     # PDFium/cache/guides/fonts checks"
	@echo "make parity     # page map + layout audit against the paper reference"
	@echo "make visual     # pixel diff against the committed visual baseline"
	@echo "make visual-capture # replace the visual baseline with the current PDF"
	@echo "make test       # cargo test"
	@echo "make fmt        # cargo fmt --check"
	@echo "make clippy     # clippy -D warnings"
	@echo "make analyze    # source inventory (SOURCES=...)"
	@echo "make pages      # low-text page screenshots (SOURCE=...)"
	@echo "make scaffold   # new guide (SLUG=... TITLE=... SOURCES=...)"

build:
	cargo run -p folisift-cli -- build --guide $(GUIDE)

check:
	cargo run -p folisift-cli -- build --guide $(GUIDE) --check

verify: check audit
	@pdfinfo $(OUTPUT) | grep -E '^(Title|Author|Pages|Page size|Tagged):'

audit:
	cargo run -p folisift-cli -- audit --pdf $(OUTPUT) --expected-pages 22

doctor:
	cargo run -p folisift-cli -- doctor

parity: check
	./scripts/check-parity.sh $(OUTPUT)

visual: build
	cargo run -p folisift-cli -- visual compare \
		--reference $(VISUAL_BASELINE) --current $(OUTPUT) \
		--pages "$(VISUAL_PAGES)" --out data/visual \
		--threshold 1.0 --ratio-threshold 0.001 --fail-on-diff

visual-capture: build
	@mkdir -p $(dir $(VISUAL_BASELINE))
	cp $(OUTPUT) $(VISUAL_BASELINE)
	@echo "visual baseline updated: $(VISUAL_BASELINE)"

test:
	cargo test --workspace

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

analyze:
	cargo run -p folisift-cli -- analyze $(addprefix --source ,$(SOURCES)) \
		--out $(DATA)/source.json --md $(DATA)/source.md

parity-capture:
	./scripts/capture-reference.sh

pages:
	cargo run -p folisift-cli -- render-pages --source $(SOURCE) \
		--pages "$(PAGES)" --out $(DATA)/pages

scaffold:
	cargo run -p folisift-cli -- scaffold --slug $(SLUG) --title "$(TITLE)" \
		--course "$(COURSE)" $(addprefix --source ,$(SOURCES))
