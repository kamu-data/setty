###############################################################################
# Lint
###############################################################################

.PHONY: lint
lint:
	cargo fmt --check
	cargo deny check --hide-inclusion-graph
	cargo clippy -p setty-derive -p setty --all-targets --all-features -- -D warnings


###############################################################################
# Lint (with fixes)
###############################################################################

.PHONY: lint-fix
lint-fix:
	cargo clippy -p setty-derive -p setty --all-targets --all-features --fix --allow-dirty --allow-staged --broken-code
	cargo fmt --all

###############################################################################
# Test
###############################################################################

.PHONY: test
test:
	cargo test -p setty --all-features
	cargo run -p example-deprecation
	cargo run -p example-jsonschema
	cargo run -p example-toml
	cargo run -p example-validate
