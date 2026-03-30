.PHONY: dev build test fmt lint clean

dev:
	npm run dev &
	cargo tauri dev

build:
	npm run build
	cargo tauri build

test:
	cargo test --manifest-path src-tauri/Cargo.toml
	npm test --if-present

fmt:
	cargo fmt --manifest-path src-tauri/Cargo.toml
	npm run lint

lint:
	cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
	npm run lint

clean:
	cargo clean --manifest-path src-tauri/Cargo.toml
	rm -rf node_modules dist
