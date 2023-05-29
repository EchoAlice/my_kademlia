test:
	cargo test
fmt:
    cargo fmt --all
lint: fmt
    cargo clippy --all-targets
build:
    cargo build --all-targets
run-ci: lint build test
run-test-with-logs TEST:
    cargo test {{TEST}} -- --nocapture