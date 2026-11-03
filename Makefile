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
VISUAL_PAGES ?= 1,4,8,12,18,23

.PHONY: help build check verify audit dep-audit toc-check reading-order-check component-contrast-check component-contrast-strict production-report revision revision-check type-check palette-check style-audit exam-check genre-check genre-visual genre-capture d2-pdf-check d2-pdf-capture package-check doctor parity parity-capture visual visual-capture diagram diagram-status package test fmt clippy analyze pages scaffold offline-exam

help:
	@echo "make build      # guide PDF"
	@echo "make check      # build + coverage gate"
	@echo "make verify     # check + layout audit + pdfinfo"
	@echo "make audit      # layout audit of the built PDF"
	@echo "make dep-audit  # RustSec dependency audit"
	@echo "make toc-check  # TOC page-number links (dev: PyMuPDF)"
	@echo "make reading-order-check # tagged structure and reading order (dev: PyMuPDF)"
	@echo "make component-contrast-check # rendered component contrast report (dev: PyMuPDF)"
	@echo "make production-report # MCP prepare/build report smoke"
	@echo "make type-check # body/code/line typography gate"
	@echo "make revision-check # revision + organiser build gates"
	@echo "make palette-check # palette contrast + grayscale proof"
	@echo "make style-audit # render and audit every style preview"
	@echo "make genre-check # notes/workbook/reference preview + baselines"
	@echo "make d2-pdf-check # per-kind D2 sequence/class/ERD PDF baseline"
	@echo "make exam-check  # build, audit, TOC and hashes of the five exam PDFs"
	@echo "make offline-exam # copy only the five final PDFs to output/offline-exam"
	@echo "make package-check # package + clean-directory doctor"
	@echo "make revision   # revision sheet + one-page organiser pilots"
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
	cargo run -p folisift-cli -- audit --pdf $(OUTPUT) --expected-pages 23 --strict

dep-audit:
	cargo audit

toc-check:
	python3 scripts/check-toc-links.py $(OUTPUT)

reading-order-check:
	python3 scripts/check-reading-order.py \
	  output/PD0-PD1_Calisma_Ozeti.pdf \
	  output/PD2_Calisma_Ozeti.pdf \
	  output/PD3_Calisma_Ozeti.pdf \
	  output/PD2_Revizyon.pdf \
	  output/PD2_LISP_Bilgi_Organizatoru.pdf

component-contrast-check:
	python3 scripts/check-component-contrast.py \
	  output/PD0-PD1_Calisma_Ozeti.pdf \
	  output/PD2_Calisma_Ozeti.pdf \
	  output/PD3_Calisma_Ozeti.pdf \
	  output/PD2_Revizyon.pdf \
	  output/PD2_LISP_Bilgi_Organizatoru.pdf

component-contrast-strict:
	python3 scripts/check-component-contrast.py --strict \
	  output/PD0-PD1_Calisma_Ozeti.pdf \
	  output/PD2_Calisma_Ozeti.pdf \
	  output/PD3_Calisma_Ozeti.pdf \
	  output/PD2_Revizyon.pdf \
	  output/PD2_LISP_Bilgi_Organizatoru.pdf

OFFLINE_DIR ?= output/offline-exam

offline-exam:
	rm -rf $(OFFLINE_DIR)
	mkdir -p $(OFFLINE_DIR)
	cp output/PD0-PD1_Calisma_Ozeti.pdf $(OFFLINE_DIR)/
	cp output/PD2_Calisma_Ozeti.pdf $(OFFLINE_DIR)/
	cp output/PD3_Calisma_Ozeti.pdf $(OFFLINE_DIR)/
	cp output/PD2_Revizyon.pdf $(OFFLINE_DIR)/
	cp output/PD2_LISP_Bilgi_Organizatoru.pdf $(OFFLINE_DIR)/
	@ls -lh $(OFFLINE_DIR)

production-report:
	python3 scripts/check-production-report.py

revision:
	cargo run -p folisift-cli -- build --guide pd2-rev --check
	cargo run -p folisift-cli -- build --guide pd2-ko --check

revision-check: revision
	@echo "revision/organiser gates passed"

type-check:
	cargo build -q -p folisift-cli
	python3 scripts/check-type-scale.py

palette-check:
	python3 scripts/check-palettes.py

style-audit:
	./scripts/check-style-previews.sh

exam-check:
	./scripts/check-exam-set.sh

genre-check: genre-visual

genre-visual:
	cargo build -q -p folisift-cli
	./target/debug/folisift styles preview notes --language en --out output/style-preview-notes-en.pdf
	./target/debug/folisift visual compare --reference tests/baselines/genres/notes.pdf --current output/style-preview-notes-en.pdf --pages 1,2,3,4 --out data/visual/genres/notes --threshold 1.0 --ratio-threshold 0.001 --fail-on-diff
	./target/debug/folisift styles preview workbook --language en --out output/style-preview-workbook-en.pdf
	./target/debug/folisift visual compare --reference tests/baselines/genres/workbook.pdf --current output/style-preview-workbook-en.pdf --pages 1,2,3,4,5 --out data/visual/genres/workbook --threshold 1.0 --ratio-threshold 0.001 --fail-on-diff
	./target/debug/folisift styles preview reference --language en --out output/style-preview-reference-en.pdf
	./target/debug/folisift visual compare --reference tests/baselines/genres/reference.pdf --current output/style-preview-reference-en.pdf --pages 1,2,3,4 --out data/visual/genres/reference --threshold 1.0 --ratio-threshold 0.001 --fail-on-diff
	./target/debug/folisift styles preview revision --language en --out output/style-preview-revision-en.pdf
	./target/debug/folisift visual compare --reference tests/baselines/genres/revision.pdf --current output/style-preview-revision-en.pdf --pages 1,2 --out data/visual/genres/revision --threshold 1.0 --ratio-threshold 0.001 --fail-on-diff

d2-pdf-check:
	./scripts/check-d2-pdf.sh

d2-pdf-capture:
	UPDATE_D2_PDF_BASELINE=1 ./scripts/check-d2-pdf.sh

genre-capture:
	cargo build -q -p folisift-cli
	mkdir -p tests/baselines/genres
	./target/debug/folisift styles preview notes --language en --out output/style-preview-notes-en.pdf
	cp output/style-preview-notes-en.pdf tests/baselines/genres/notes.pdf
	./target/debug/folisift styles preview workbook --language en --out output/style-preview-workbook-en.pdf
	cp output/style-preview-workbook-en.pdf tests/baselines/genres/workbook.pdf
	./target/debug/folisift styles preview reference --language en --out output/style-preview-reference-en.pdf
	cp output/style-preview-reference-en.pdf tests/baselines/genres/reference.pdf
	./target/debug/folisift styles preview revision --language en --out output/style-preview-revision-en.pdf
	cp output/style-preview-revision-en.pdf tests/baselines/genres/revision.pdf

package-check: package
	./scripts/check-package.sh

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
