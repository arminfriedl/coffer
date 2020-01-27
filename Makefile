CURDIR=$(shell pwd)

default:
	cargo build

release:
	cargo build --release

publish:
	podman pull clux/muslrust
	podman run -v .:/volume:Z --rm -t clux/muslrust cargo build --release
	strip target/x86_64-unknown-linux-musl/release/coffer-server
	strip target/x86_64-unknown-linux-musl/release/coffer-client
	strip target/x86_64-unknown-linux-musl/release/coffer-companion

.PHONY: default release publish
