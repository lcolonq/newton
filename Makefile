CARGO_OPTS = --config 'patch."https://github.com/lcolonq/teleia".teleia.path="../teleia/crates/teleia"'

.PHONY: build

build:
	@mkdir -p .tmp/
	@cp Cargo.lock .tmp/Cargo.lock
	@cargo build $(CARGO_OPTS)
	@cp .tmp/Cargo.lock Cargo.lock
