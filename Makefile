# The NX remote desktop preload aborts rustfmt/clippy on this host.
unexport LD_PRELOAD

GUIDE ?= pd0-pd1
SOURCES ?= sources/PD0.pdf sources/PD1.pdf
SOURCE ?= sources/PD0.pdf
PAGES ?= 1
DATA ?= guides/$(GUIDE)/data
OUTPUT ?= output/PD0-PD1_Calisma_Ozeti.pdf
VISUAL_BASELINE ?= tests/baselines/pd0-pd1.pdf
CLEAN_BASELINE ?= tests/baselines/clean-preview.pdf
VISUAL_PAGES ?= 1,4,8,12,18,22

.PHONY: help build check verify audit toc-check production-report doctor parity parity-capture visual visual-capture diagram diagram-status package test fmt clippy analyze pages scaffold

help:
	@echo "make build      # guide PDF"
	@echo "make check      # build + coverage gate"
	@echo "make verify     # check + layout audit + pdfinfo"
	@echo "make audit      # layout audit of the built PDF"
	@echo "make toc-check  # TOC page-number links (dev: PyMuPDF)"
	@echo "make production-report # MCP prepare/build report smoke"
	@echo "make doctor     # PDFium/cache/guides/fonts checks"
	@echo "make parity     # page map + layout audit against the paper reference"
	@echo "make diagram    # render all D2 diagram assets (DIAGRAM_ID=... for one)"
	@echo "make diagram-status # check diagram specs/manifests without D2"
	@echo "make visual     # pixel diff against the committed visual baseline"
	@echo "make visual-capture # replace the visual baseline with the current PDF"
	@echo "make package    # Linux release archive + SHA-256"
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
	cargo run -p folisift-cli -- audit --pdf $(OUTPUT) --expected-pages 22 --strict

toc-check:
	python3 scripts/check-toc-links.py $(OUTPUT)

production-report:
	python3 scripts/check-production-report.py

doctor:
	cargo run -p folisift-cli -- doctor

diagram:
	cargo run -p folisift-cli -- diagram build --guide $(GUIDE) $(if $(DIAGRAM_ID),--id $(DIAGRAM_ID),--all)

diagram-status:
	cargo run -p folisift-cli -- diagram status --guide $(GUIDE)

parity: check
	./scripts/check-parity.sh $(OUTPUT)

visual: build
	cargo run -p folisift-cli -- visual compare \
		--reference $(VISUAL_BASELINE) --current $(OUTPUT) \
		--pages "$(VISUAL_PAGES)" --out data/visual \
		--threshold 1.0 --ratio-threshold 0.001 --fail-on-diff
	cargo run -p folisift-cli -- styles preview clean --language en \
		--out output/style-preview-clean-en.pdf
	cargo run -p folisift-cli -- visual compare \
		--reference $(CLEAN_BASELINE) \
		--current output/style-preview-clean-en.pdf \
		--pages 1,2,3,4 --out data/visual/clean \
		--threshold 1.0 --ratio-threshold 0.001 --fail-on-diff

visual-capture: build
	@mkdir -p $(dir $(VISUAL_BASELINE)) $(dir $(CLEAN_BASELINE))
	cp $(OUTPUT) $(VISUAL_BASELINE)
	cargo run -p folisift-cli -- styles preview clean --language en \
		--out output/style-preview-clean-en.pdf
	cp output/style-preview-clean-en.pdf $(CLEAN_BASELINE)
	@echo "visual baselines updated: $(VISUAL_BASELINE), $(CLEAN_BASELINE)"

package:
	./scripts/package.sh

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
