default:
    @just --list

# Aliases
alias d  := dev
alias dw := dev-web
alias l  := lint
alias f  := fmt
alias t  := test
alias c  := typecheck

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

# Typecheck
[group('typecheck')]
[doc("Typecheck all (cargo + svelte)")]
typecheck: typecheck-rust typecheck-web

[group('typecheck')]
[private]
typecheck-rust:
    cargo check

[group('typecheck')]
[private]
typecheck-web:
    pnpm svelte-check

# Check
[group('ci')]
[doc("Full check: typecheck + lint + format")]
check: typecheck lint fmt-check

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
    pnpm lint

[group('lint')]
[doc("Lint & fix all")]
lint-fix: lint-fix-rust lint-fix-web

[group('lint')]
[private]
lint-fix-rust:
    cargo clippy --workspace --fix --allow-dirty

[group('lint')]
[private]
lint-fix-web:
    pnpm lint:fix

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
    pnpm format

[group('format')]
[doc("Check formatting (no write)")]
fmt-check: fmt-check-rust fmt-check-web

[group('format')]
[private]
fmt-check-rust:
    cargo fmt --all -- --check

[group('format')]
[private]
fmt-check-web:
    pnpm format:check

# Test
[group('test')]
[doc("Run all tests")]
test: test-rust test-web

[group('test')]
[doc("Run app-core tests only")]
test-core:
    cargo test -p app-core -- --skip export_bindings

[group('test')]
[private]
test-rust:
    cargo test -- --skip export_bindings

[group('test')]
[private]
test-web:
    @echo "test-web: not configured yet"

# Gen
[group('gen')]
[doc("Generate TS types from Rust models")]
gen-types:
    TS_RS_EXPORT_DIR="$(pwd)/src/lib/types/bindings" cargo test -p app-core export_bindings

# Dev utilities
[group('dev')]
[doc("Open local SQLite database in VS Code")]
open-db:
    codium "$HOME/Library/Application Support/dev.guillevc.satfolio/satfolio.db"

[group('dev')]
[doc("Delete local SQLite database")]
reset-db:
    rm -f ~/Library/Application\ Support/dev.guillevc.satfolio/satfolio.db
    @echo "Database deleted"

[group('dev')]
[doc("Run shadcn-svelte CLI (e.g. just shadcn add button)")]
shadcn *args:
  -pnpm dlx shadcn-svelte@latest {{args}}
