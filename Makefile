default:
	cargo build

release:
	cargo build --release

publish:
	docker pull clux/muslrust
	docker run -v $PWD:/volume --rm -t clux/muslrust cargo build --release

.PHONY: default release publish
