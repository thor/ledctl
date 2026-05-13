.PHONY: universal clean

universal:
	rustup target add aarch64-apple-darwin x86_64-apple-darwin
	cargo build --release --target aarch64-apple-darwin
	cargo build --release --target x86_64-apple-darwin
	lipo -create \
	    -output target/ledctl \
	    target/aarch64-apple-darwin/release/ledctl \
	    target/x86_64-apple-darwin/release/ledctl
	@echo "Universal binary at target/ledctl"
	@lipo -info target/ledctl

clean:
	cargo clean
	rm -f target/ledctl
