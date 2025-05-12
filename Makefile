build:
	cargo build
run:
	cargo run
clippy:
	cargo clippy -- -D warnings
fmt:
	cargo fmt --check
test:
	cargo test
doc:
	cargo doc
	cargo doc --open
2048:
	cargo run ./images/2048.obj
rogue:
	cargo run ./images/rogue.obj
