ifdef GIT_ROOT
else
GIT_ROOT := $(shell git rev-parse --show-toplevel 2>/dev/null)
endif

MK_DIR := $(GIT_ROOT)/.make

include ekgf-make.mk

.PHONY: build-lambda-invoke
build-lambda-invoke:
	$(MAKE) -C $(GIT_ROOT)/crate/ekg-lfn-invoke build

.PHONY: build-lambda-load
build-lambda-load:
	$(MAKE) -C $(GIT_ROOT)/crate/ekg-lfn-load build

.PHONY: build-lambda-check
build-lambda-check: poetry-check
	@echo "build-lambda-check"
	cd $(GIT_ROOT)/lambda/check && $(POETRY_BIN) build

.PHONY: build
build: build-lambda-invoke build-lambda-load build-lambda-check