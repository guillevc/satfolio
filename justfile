default:
    @just --list

# Aliases
alias d := dev
alias l := lint
alias f := fmt
alias t := test
alias c := check

# Dev
[group('dev')]
[doc("Run full Tauri desktop app")]
dev:
    pnpm tauri dev

[group('dev')]
[doc("Run Vite dev server only (:5173)")]
dev-web:
    pnpm dev

# Build
[group('build')]
[doc("Build native desktop bundle")]
build:
    pnpm tauri build

[group('build')]
[doc("Build frontend only (tsc + vite)")]
build-web:
    pnpm build

# Check
[group('check')]
[doc("Check all (cargo + typecheck)")]
check: check-rust check-web

[group('check')]
[private]
check-rust:
    cargo check

[group('check')]
[private]
check-web:
    pnpm check

# Lint
[group('lint')]
[doc("Lint all")]
lint: lint-rust lint-web

[group('lint')]
[private]
lint-rust:
    cargo clippy --workspace

[group('lint')]
[private]
lint-web:
    @echo "lint-web: not configured yet"

# Format
[group('format')]
[doc("Format all")]
fmt: fmt-rust fmt-web

[group('format')]
[private]
fmt-rust:
    cargo fmt --all

[group('format')]
[private]
fmt-web:
    @echo "fmt-web: not configured yet"

# Test
[group('test')]
[doc("Run all tests")]
test: test-rust test-web

[group('test')]
[doc("Run app-core tests only")]
test-core:
    cargo test -p app-core

[group('test')]
[private]
test-rust:
    cargo test

[group('test')]
[private]
test-web:
    @echo "test-web: not configured yet"

# Examples
[group('examples')]
[doc("Parse a CSV file")]
parse-csv:
    cargo run -p app-core --example parse_csv
