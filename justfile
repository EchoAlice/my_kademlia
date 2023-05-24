test:
	cargo test
fmt:
    cargo fmt --all
lint: fmt
    cargo clippy --all-targets
build:
    cargo build --all-targets
run-ci: lint build test


# Run "cargo test add_node -- --nocapture" to see that node was added to routing table.