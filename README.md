<h1>
  satfolio <img src="https://img.shields.io/github/actions/workflow/status/guillevc/satfolio/ci.yaml?style=flat&labelColor=black&label=ci" alt="CI" />
</h1>

A desktop app to track your Bitcoin portfolio. Import your trade history from Kraken or Coinbase, and see your position, break-even price, and P&L — all stored locally on your machine.

<img src="docs/dashboard.png" alt="satfolio dashboard" />

## Features

- **Import trades** from Kraken and Coinbase CSV ledger exports, with automatic duplicate detection
- **Dashboard** showing current BTC price, break-even price, position value, and unrealized P&L
- **Price chart** with daily candles and your trade history overlaid
- **Trade history** table with per-trade cost basis, break-even price, and realized P&L
- **Private by default** — no accounts, no analytics, no telemetry. Data stays in a local SQLite database. The only network call is fetching the current BTC price from Kraken's public API.

**Planned:** multi-currency support (EUR, USD, GBP).

## Install

Download the latest release from the [Releases](https://github.com/guillevc/satfolio/releases) page.

| Platform              | File                             |
| --------------------- | -------------------------------- |
| macOS (Apple Silicon) | `Satfolio_<version>_aarch64.dmg` |
| macOS (Intel)         | `Satfolio_<version>_x64.dmg`     |
| Linux (x64)           | `.deb`, `.rpm`, or `.AppImage`   |

### macOS

> [!NOTE]
> macOS shows a warning because Satfolio isn't signed through Apple's paid developer program. The app is open source and every release is verifiably built from this repo — see [Security & trust](#security--trust).

1. Open the `.dmg` and drag Satfolio to **Applications**
2. Try to open Satfolio — macOS will show a warning and block it
3. Open **System Settings → Privacy & Security**
4. Under Security, click **Open Anyway**
5. Enter your login password and click **OK**

This is only needed once — after that, Satfolio opens normally. See [Apple's support page](https://support.apple.com/guide/mac-help/open-a-mac-app-from-an-unknown-developer-mh40616/mac) for more details.

Alternatively, run this in Terminal:

```sh
xattr -d com.apple.quarantine /Applications/Satfolio.app
```

## Security & trust

This project is free and open source. Apple's Developer Program costs 99€/year, so instead of paying for a code signature, every release is built transparently in public CI and cryptographically signed via [Sigstore](https://www.sigstore.dev).

Release artifacts include SHA-256 checksums and [build provenance attestations](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations) so you can verify exactly how each binary was built.

```sh
# verify your download hasn't been tampered with
shasum -a 256 --check SHA256SUMS.txt

# verify the binary was built from this repo's source code (requires GitHub CLI)
gh attestation verify <filename> --owner guillevc
```

## Build from source

```sh
git clone https://github.com/guillevc/satfolio.git
cd satfolio
mise install      # install toolchain (node, pnpm, rust, just)
just install      # install frontend dependencies
just build        # build the Tauri app
```

Requires [mise](https://mise.jdx.dev) (or manually: just, Rust, Node.js, pnpm) and [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/).

## Development

After cloning and running `mise install` and `just install` (see [Build from source](#build-from-source)):

```sh
just dev       # run in development mode
just check     # typecheck + lint + format check
just test      # run all tests
```

Run `just` to see all available recipes.

## License

[GPL-3.0](LICENSE)
