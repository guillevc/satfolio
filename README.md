# Satfolio

A desktop app for tracking Bitcoin trades and portfolio performance. Import CSV ledgers from Kraken or Coinbase, visualize price history with your trades overlaid, and see your position, break-even price, and P&L at a glance.

Built with [Tauri 2](https://v2.tauri.app), Svelte 5, and Rust.

## Features

- **Import trades** from Kraken and Coinbase CSV exports, with automatic duplicate detection
- **Dashboard** with current BTC price, break-even price, position value, and unrealized P&L
- **Price chart** with daily candles and trade history overlay
- **Trade history** table with per-trade break-even price and realized P&L
- **Multi-currency** support (EUR, USD, GBP)
- **Local-only** — your data stays on your machine in a SQLite database

## Install

Download the latest release from the [Releases](https://github.com/guillevc/satfolio/releases) page:

| Platform              | File                  |
| --------------------- | --------------------- |
| macOS (Apple Silicon) | `.dmg`                |
| macOS (Intel)         | `.dmg`                |
| Linux (x64)           | `.deb` or `.AppImage` |

### macOS installation note

Satfolio is not signed with an Apple Developer certificate, so macOS will show a security warning on first launch. This is normal for independent open-source software.

1. Open the `.dmg` and drag Satfolio to Applications
2. Try to open Satfolio — macOS will block it
3. Go to **System Settings > Privacy & Security**
4. Scroll to **Security** — you'll see a message about Satfolio being blocked
5. Click **Open Anyway** and confirm with your password
6. This is only needed once

Alternatively, for technical users:

```sh
xattr -d com.apple.quarantine /Applications/Satfolio.app
```

## Security & trust

Satfolio is local-only. There are no analytics, no telemetry, and no network calls except fetching the current BTC price from Kraken's public API.

### Why no Apple code signing?

Apple's Developer Program costs $99/year and requires identity verification. Satfolio is a free, open-source tool. Instead of paying for a signature, every release is built transparently in public CI with cryptographic provenance you can verify yourself.

### Verifying a release

Every release artifact has:

- **SHA-256 checksums** in `SHA256SUMS.txt` attached to the release
- **Build provenance attestations** signed via Sigstore through GitHub Actions ([SLSA Build L2](https://slsa.dev))

```sh
# Check file integrity
shasum -a 256 --check SHA256SUMS.txt

# Verify build provenance (requires GitHub CLI)
gh attestation verify <filename> --owner guillevc
```

## Build from source

If you prefer not to trust pre-built binaries:

```sh
git clone https://github.com/guillevc/satfolio.git
cd satfolio
just install
just build
```

Requires: Rust (see `rust-toolchain.toml`), Node.js, pnpm, and [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/).

## Development

```sh
just install   # Install dependencies
just dev       # Run Tauri desktop app
just check     # Typecheck + lint + format check
just test      # Run all tests
```

Run `just` to see all available recipes.

## License

MIT
