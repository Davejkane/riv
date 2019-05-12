all: build cpy_bin cpy_ttf

build:
	cargo build --release

cpy_bin:
	cp ./target/release/riv /usr/local/bin/riv

cpy_ttf:
	cp ./resources/Roboto-Medium.ttf /usr/local/share/Roboto-Medium.ttf

check: fmt ccheck test clippy

fmt:
	cargo fmt --all -- --check

ccheck:
	cargo check

test:
	cargo test --all

clippy:
	cargo clippy
