GUIDE ?= guides/pd0-pd1
SOURCES ?= ../../sources/PD0.pdf ../../sources/PD1.pdf
SOURCE ?= PD0.pdf
PY ?= python3

.PHONY: help build check verify inventory pages clean-pages scaffold

help:
	@echo "make build      # guide PDF"
	@echo "make check      # coverage gate + PDF"
	@echo "make verify     # check + pdfinfo"
	@echo "make inventory  # source inventory (GUIDE, SOURCES)"
	@echo "make pages      # low-text contact sheets"
	@echo "make scaffold   # yeni guide (SLUG=... TITLE=... SOURCES=...)"

build:
	$(PY) engine/build.py --guide $(GUIDE)

check:
	$(PY) engine/build.py --guide $(GUIDE) --check

verify: check
	@find output -maxdepth 1 -name '*.pdf' -printf 'PDF: %f (%s bytes)\n'

inventory:
	cd $(GUIDE) && $(PY) ../../engine/tools/analyze_sources.py $(SOURCES) \
		--out data/source.json --md data/source.md

pages:
	cd $(GUIDE) && $(PY) ../../engine/tools/prepare_pages.py $(firstword $(SOURCES)) \
		--source-json data/source.json --filename $(SOURCE) \
		--out data/pages/$(basename $(SOURCE) .pdf) --cols 2 --rows 2

clean-pages:
	rm -rf $(GUIDE)/data/pages

scaffold:
	$(PY) engine/tools/scaffold.py --slug $(SLUG) --title "$(TITLE)" \
		--course "$(COURSE)" --sources $(SOURCES)
