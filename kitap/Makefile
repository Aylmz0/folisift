SHELL := /bin/bash
PY ?= python3
SLUG ?= pd0-pd1
CONFIG ?= config/$(SLUG).json

.PHONY: help analyze pages check build pilot clean-data

help:
	@echo "make analyze   # PD0 + PD1 kaynak envanteri"
	@echo "make pages     # PD0 düşük metinli sayfalar"
	@echo "make check     # başlık kapsamı"
	@echo "make build     # PDF"
	@echo "make pilot     # check + build"

analyze:
	$(PY) tools/analyze_sources.py ../sources/PD0.pdf ../sources/PD1.pdf \
		--out data/$(SLUG).source.json \
		--md data/$(SLUG).source.md

pages:
	$(PY) tools/prepare_pages.py ../sources/PD0.pdf \
		--source-json data/$(SLUG).source.json --filename PD0.pdf \
		--out data/pages/pd0 --cols 2 --rows 2

check:
	$(PY) tools/check_coverage.py \
		--source-json data/$(SLUG).source.json \
		--html pilot.html \
		--out data/$(SLUG).coverage.md

build:
	$(PY) build.py --config $(CONFIG)

pilot: check build
