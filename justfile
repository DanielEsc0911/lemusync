# LemuSync task runner. `just --list` shows every recipe.
# On Windows, run from Git Bash (just uses `sh`).

default:
    @just --list

# Format Rust and Dart
fmt:
    cargo fmt --all
    cd app && dart format lib test

# Lints and repo checks (no tests)
lint:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo deny check
    cargo xtask layers
    cargo xtask docs-check
    cargo xtask i18n-check
    cd app && flutter gen-l10n
    cd app && dart format --output=none --set-exit-if-changed lib test
    cd app && flutter analyze

# Rust and Flutter tests
test:
    cargo test --workspace
    cd app && flutter test

# The Linux Secret Service client on a throwaway D-Bus session (never yours); cleans up after
# itself and after any earlier crashed run. Needs dbus-run-session and gnome-keyring-daemon.
secret-service-test dir="target":
    scripts/isolated-secret-service.sh {{dir}}

# Everything CI runs; must pass before every commit
check: lint test

# Budgets (reliability §Budgets): engine benches, release sizes, the isolated daemon over
# `idle` seconds (Linux), then the CI regression check
bench idle="600":
    cargo bench -p lemusync-engine --bench budgets
    cargo build --release -p lemusync-cli -p lemusync-daemon
    ls -l target/release/lemusync target/release/lemusyncd
    cargo xtask budgets {{idle}}
    cargo xtask budgets-check
