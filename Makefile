CARGO_OPTS = --config 'patch."https://github.com/lcolonq/teleia".teleia.path="../teleia/crates/teleia"' \
  --config 'patch."https://github.com/lcolonq/teleia".teleia_macros.path="../teleia/crates/teleia_macros"'

.PHONY: build run rerun

build:
	@mkdir -p .tmp/
	@cp Cargo.lock .tmp/Cargo.lock
	@cargo build $(CARGO_OPTS)
	@cp .tmp/Cargo.lock Cargo.lock
