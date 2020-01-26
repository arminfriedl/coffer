CURDIR=$(shell pwd)

default:
	cargo build

release:
	cargo build --release

publish:
	podman pull clux/muslrust
	podman run -v .:/volume --rm -t clux/muslrust cargo build --release

.PHONY: default release publish
