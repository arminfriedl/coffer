CURDIR=$(shell pwd)

default:
	cargo build

release:
	cargo build --release

publish:
	docker pull clux/muslrust
	docker run -v $(CURDIR):/volume --rm -t clux/muslrust cargo build --release

.PHONY: default release publish
