<h1>
  satfolio <img src="https://img.shields.io/github/actions/workflow/status/guillevc/satfolio/ci.yaml?style=flat&labelColor=black&label=ci" alt="CI" /></a>
</h1>

A desktop app to track your Bitcoin portfolio. Import your trade history from Kraken or Coinbase, and see your position, break-even price, and P&L — all stored locally on your machine.

<img src="docs/dashboard.png" alt="satfolio dashboard" />

## Features

- **Import trades** from Kraken and Coinbase CSV ledger exports, with automatic duplicate detection
- **Dashboard** showing current BTC price, break-even price, position value, and unrealized P&L
- **Price chart** with daily candles and your trade history overlaid
- **Trade history** table with per-trade cost basis, break-even price, and realized P&L
- ~~**Multi-currency** — track in EUR, USD, or GBP~~
- **Private by default** — no accounts, no analytics, no telemetry. Data stays in a local SQLite database. The only network call is fetching the current BTC price from Kraken's public API.

## Install

Download the latest release from the [Releases](https://github.com/guillevc/satfolio/releases) page.

| Platform              | File                           |
| --------------------- | ------------------------------ |
| macOS (Apple Silicon) | `.dmg`                         |
| macOS (Intel)         | `.dmg`                         |
| Linux (x64)           | `.deb`, `.rpm`, or `.AppImage` |

### macOS installation

1. Open the `.dmg` and drag Satfolio to Applications
2. Try to open it — macOS will block it
3. Go to **System Settings → Privacy & Security**
4. Scroll down — you'll see a message about Satfolio being blocked
5. Click **Open Anyway** and confirm with your password

This is only needed once. Alternatively:

```sh
xattr -d com.apple.quarantine /Applications/Satfolio.app
```

## Security & trust

This project is free and open source. Apple's Developer Program costs 99€/year, so instead of paying for a code signature, every release is built transparently in public CI with cryptographic provenance you can verify yourself.

Every release includes SHA-256 checksums (`SHA256SUMS.txt`) and [build provenance attestations](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations) signed via Sigstore through GitHub Actions, achieving [SLSA Build Level 2](https://slsa.dev).

```sh
# Check file integrity
shasum -a 256 --check SHA256SUMS.txt

# Verify build provenance (requires GitHub CLI)
gh attestation verify <filename> --owner guillevc
```

## Build from source

```sh
git clone https://github.com/guillevc/satfolio.git
cd satfolio
just install   # install frontend dependencies
just build     # build the Tauri app
```

Requires just, Rust (see `rust-toolchain.toml`), Node.js, pnpm, and [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/).

## Development

```sh
just dev       # run in development mode
just check     # typecheck + lint + format check
just test      # run all tests
```

Run `just` to see all available recipes.

## License

[GPL-3.0](LICENSE)
