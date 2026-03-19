default:
    @just --list

# Aliases
alias d  := dev
alias du := dev-ui
alias l  := lint
alias f  := fmt
alias t  := test
alias c  := check
alias i  := install

# Setup
[group('setup')]
[doc("Install all dependencies")]
install:
    pnpm install

# Dev
[group('dev')]
[doc("Run full Tauri desktop app")]
dev:
    pnpm tauri dev

[group('dev')]
[doc("Run Vite dev server only (:5173)")]
dev-ui:
    pnpm dev

# Build
[group('build')]
[doc("Build native desktop bundle")]
build:
    pnpm tauri build

[group('build')]
[doc("Build frontend only (tsc + vite)")]
build-ui:
    pnpm build

# CI
[group('ci')]
[doc("Build Tauri app without bundling (smoke test)")]
smoke-app:
    pnpm tauri build --no-bundle

[group('ci')]
[doc("Audit all dependencies for known vulnerabilities")]
audit:
    command -v cargo-audit > /dev/null || cargo install cargo-audit --locked
    cargo audit

[group('ci')]
[doc("Full check: typecheck + lint + format")]
check: typecheck lint fmt-check

# Typecheck
[group('typecheck')]
[doc("Typecheck all (cargo + svelte)")]
typecheck: typecheck-rust typecheck-ui

[group('typecheck')]
[private]
typecheck-rust:
    cargo check

[group('typecheck')]
[private]
typecheck-ui:
    pnpm svelte-check

# Lint
[group('lint')]
[doc("Lint all")]
lint: lint-rust lint-ui

[group('lint')]
[private]
lint-rust:
    cargo clippy --workspace

[group('lint')]
[private]
lint-ui:
    pnpm lint

[group('lint')]
[doc("Lint & fix all")]
lint-fix: lint-fix-rust lint-fix-ui

[group('lint')]
[private]
lint-fix-rust:
    cargo clippy --workspace --fix --allow-dirty

[group('lint')]
[private]
lint-fix-ui:
    pnpm lint:fix

# Format
[group('format')]
[doc("Format all")]
fmt: fmt-rust fmt-ui

[group('format')]
[private]
fmt-rust:
    cargo fmt --all

[group('format')]
[private]
fmt-ui:
    pnpm format

[group('format')]
[doc("Check formatting (no write)")]
fmt-check: fmt-check-rust fmt-check-ui

[group('format')]
[private]
fmt-check-rust:
    cargo fmt --all -- --check

[group('format')]
[private]
fmt-check-ui:
    pnpm format:check

# Test
[group('test')]
[doc("Run all tests")]
test: test-core test-ui

[group('test')]
[doc("Run app-core tests only")]
test-core:
    cargo test -p app-core -- --skip export_bindings

[group('test')]
[private]
test-ui:
    pnpm test

# Gen
[group('gen')]
[doc("Generate app icons from src-tauri/app-icon.svg")]
gen-icons:
    pnpm tauri icon src-tauri/app-icon.svg
    rm -rf src-tauri/icons/ios src-tauri/icons/android

[group('gen')]
[doc("Generate TS types from Rust models")]
gen-types-core:
    TS_RS_EXPORT_DIR="$(pwd)/src/lib/types/bindings" cargo test -p app-core export_bindings

# Release
[group('release')]
[doc("Print changelog (default: unreleased, e.g. just changelog v0.1.0)")]
changelog ref='':
    git-cliff {{ if ref == "" { "--unreleased" } else { ref + "..HEAD" } }}

[group('release')]
[doc("Bump version, commit, and tag (e.g. just version 1.0.0)")]
version v:
    node -e "let p='package.json',j=JSON.parse(require('fs').readFileSync(p));j.version='{{v}}';require('fs').writeFileSync(p,JSON.stringify(j,null,2)+'\n')"
    sed -i '' '3s/version = "[^"]*"/version = "{{v}}"/' src-tauri/Cargo.toml
    sed -i '' '3s/version = "[^"]*"/version = "{{v}}"/' crates/core/Cargo.toml
    cargo generate-lockfile
    git add package.json src-tauri/Cargo.toml crates/core/Cargo.toml Cargo.lock
    git commit -m "chore: bump version to {{v}}"
    git tag "v{{v}}"
    @echo "Tagged v{{v}} — push with: git push && git push --tags"

# Clean
[group('dev')]
[doc("Purge all build caches (cargo, node_modules, vite)")]
[confirm("This will delete target/, node_modules/, and .vite cache. Continue?")]
clean:
    rm -rf target
    rm -rf node_modules
    rm -rf .vite
    @echo "All caches purged — run 'just install' to restore node_modules"

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
