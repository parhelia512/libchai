.PHONY: assets assetsdir package

all: build

%.txt:
	curl "https://assets.chaifen.app/$@" -o assets/$@

%.yaml:
	curl "https://assets.chaifen.app/$@" -o assets/$@

assets: assetsdir frequency.txt key_distribution.txt pair_equivalence.txt

assetsdir:
	mkdir -p assets

examples: # 冰雪四拼.yaml 冰雪四拼.txt 米十五笔.yaml 米十五笔.txt
	mkdir -p examples; \
	for file in 冰雪四拼.yaml 冰雪四拼.txt 米十五笔.yaml 米十五笔.txt; do \
		curl "https://assets.chaifen.app/$$file" -o examples/$$file; \
	done

package: build-macos-arm build-macos-x86 build-windows build-linux-gnu build-linux-musl
	mkdir -p package; \
	lipo -output package/chai -create target/aarch64-apple-darwin/release/chai target/x86_64-apple-darwin/release/chai
	cp target/x86_64-unknown-linux-gnu/release/chai package/chai-gnu; \
	cp target/x86_64-unknown-linux-musl/release/chai package/chai-musl; \
	cp target/x86_64-pc-windows-gnu/release/chai.exe package/; \
	cp -r README.md config.md LICENSE config.yaml elements.txt assets package/; \
	cd package; \
	rm chai.zip; \
	zip -r chai.zip *

build:
	cargo build --release --bin chai

build-macos-arm:
	cargo build --release --bin chai --target aarch64-apple-darwin

build-macos-x86:
	cargo build --release --bin chai --target x86_64-apple-darwin

build-windows:
	cargo build --release --bin chai --target x86_64-pc-windows-gnu

build-linux-gnu:
	cargo build --release --bin chai --target x86_64-unknown-linux-gnu

build-linux-musl:
	cargo build --release --bin chai --target x86_64-unknown-linux-musl

wasm:
	wasm-pack build --target web

publish:
	wasm-pack publish
