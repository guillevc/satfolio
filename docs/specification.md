# betc — BTC Break-Even Price Tracker

## Concept

A privacy-first desktop app that imports your Bitcoin transaction history from a Kraken CSV export, computes your break-even price over time, and visualizes it against the live BTC price. A personal clarity tool — understand what you paid, what you're holding, and how you're doing. Primarily a portfolio insight app, with an optional tax reporting layer for cost basis compliance.

## Tech Stack

| Layer | Choice | Rationale |
|-------|--------|-----------|
| App framework | Tauri v2 (desktop-only) | Native OS webview, ~3MB binary, <500ms startup. Mobile path available later if needed. |
| Backend | Rust | Handles all business logic: parsing, computation, price fetching, storage. |
| Frontend | Svelte 5 + Vite + TypeScript | Plain Vite, no SvelteKit — no routing or SSR needed for a single-view desktop app. |
| Charting | lightweight-charts v5 (TradingView OSS) | Used directly without wrapper. Financial-grade, canvas-rendered, tiny footprint. |
| UI components | shadcn-svelte + Tailwind CSS | Copies into project (not a dependency). Dark theme only. |
| Storage | SQLite via rusqlite | Local single-file DB for trades and computed snapshots. |

## Architecture: Rust Computes, Frontend Renders

The frontend is dumb. Rust owns all computation; the frontend receives structured data and renders it.

**Rust returns after import:**

- **Trade list** — parsed, normalized, stored in SQLite. Frontend renders the table.
- **BEP time series** — sparse, one snapshot per trade event: `{ date, bep, btc_held, total_spent, total_received }`. Frontend plots it as a stepped line.
- **Price series** — dense, one candle per day: `{ date, open, high, low, close, volume, count }`. Full OHLC stored internally; frontend uses what it needs. ~4,000 rows from 2013 to today. Negligible over Tauri's in-process IPC.
- **Dashboard stats** — current BEP, total held, total spent, total received, realized P&L total.
- **Per-sell P&L** — each sell's realized gain (weighted average), attached to the trade.
- **Tax view data** — FIFO lot matching per sell. Separate query.
- **Import summary** — counts, date range, totals, warnings.

**Frontend derives on hover / live tick:**

- `unrealized_pnl = (price - bep) × btc_held` — trivial lookup and subtraction at any date.

BEP only changes on trade events, so there's no need to pre-compute P&L at every daily price point. The chart interpolates from the sparse BEP series and the dense price series.

## Code Design & API

### Workspace Structure

```
betc/
├── Cargo.toml              # workspace root
├── crates/
│   └── core/               # app-core: all business logic, zero Tauri knowledge
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── api.rs      # orchestration layer
│       │   ├── context.rs  # Context struct (holds DB connection)
│       │   ├── models.rs   # data types
│       │   ├── errors.rs   # error + result types
│       │   ├── parser.rs   # Kraken CSV → trades
│       │   ├── engine.rs   # BEP computation
│       │   ├── tax.rs      # cost basis lot matching
│       │   ├── price.rs    # bundled CSV loading, Kraken OHLC + ticker HTTP
│       │   └── db.rs       # SQLite persistence + migrations
│       ├── fixtures/       # reduced test data (price CSVs, sample ledger)
│       └── examples/
├── src-tauri/               # thin Tauri shell
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   └── lib.rs           # #[tauri::command] one-liners
│   ├── resources/prices/    # bundled full price CSVs (XBTEUR, XBTGBP, XBTUSD)
│   └── tauri.conf.json
├── src/                     # Svelte frontend
├── package.json
└── vite.config.ts
```

### Core Modules

One file per module, flat structure, no folders or `mod.rs`. Modern Rust convention. If a module grows too large (e.g. `parser` when adding exchange formats), promote it to a folder then:

```
crates/core/src/
├── lib.rs          # module declarations
├── api.rs          # orchestration
├── context.rs      # Context struct (DB connection holder)
├── models.rs       # all data types
├── errors.rs       # all error + result types
├── parser.rs       # Kraken CSV → trades
├── engine.rs       # BEP math
├── tax.rs          # cost basis lots
├── price.rs        # bundled CSV + Kraken HTTP (OHLC gap-fill, live ticker)
└── db.rs           # SQLite persistence + migrations
```

**Module visibility** — `api`, `context`, `models`, and `errors` are public. Everything else is crate-internal:

```rust
// lib.rs
pub mod api;       // public — the only way into core's logic
pub mod context;   // public — Context appears in api signatures
pub mod models;    // public — data types appear in api signatures
pub mod errors;    // public — error and result types

mod parser;        // private — only reachable within this crate
mod engine;        // private
mod tax;           // private
mod price;         // private
mod db;            // private
```

Four public modules: `models` for data types, `errors` for error and result types, `context` for the DB connection holder, `api` for functions. Private modules are visible to all siblings within the crate (`api.rs` can `use crate::parser`), but invisible from outside. The Tauri crate can only reach `app_core::api`, `app_core::context`, `app_core::models`, and `app_core::errors`.

```
api        orchestration (composes pure + IO)
├── parser    pure: Kraken CSV → trades
├── engine    pure: trades → BEP, stats
├── tax       pure: trades → tax lots
├── db        IO: SQLite reads/writes + migrations
└── price     IO: bundled CSV + HTTP (OHLC gap-fill, live ticker)
```

`parser`, `engine`, and `tax` are pure functions — no state, no IO, trivially testable. `db` wraps SQLite with versioned migrations. `price` handles bundled CSV loading, Kraken OHLC API for gap-filling, and Kraken Ticker for live price. `api` composes them into workflows.

### Naming Convention

- **LedgerEntry**: raw CSV row (anything Kraken logged — deposits, withdrawals, staking, trades). Internal to `parser`, never exposed.
- **Trade**: a normalized buy or sell. Uses `AssetAmount` (amount + asset) for `spent`, `received`, and `fee` fields. Direction determined by `side_for(pair)`.
- **Candle**: daily OHLC price data. Full `{ date, open, high, low, close, volume, count }` stored internally; consumers use the fields they need.
- **Asset**: universal identifier for any asset — `Btc`, `Eur`, `Gbp`, `Usd`, `Other(String)`. Handles Kraken's Z-prefixed codes (`ZEUR` → `Eur`). No separate `Currency` enum — `Asset` covers both crypto and fiat.
- **AssetPair**: a `{ base, quote }` pair (e.g. BTC/EUR). Used by the engine to determine trade direction.
- **Context**: holds the SQLite connection and the user's selected quote currency. Passed to `api` functions. Created via `Context::open(path, quote)`.

### Signatures

```rust
// lib.rs
pub mod api;
pub mod context;
pub mod models;
pub mod errors;

mod db;
mod engine;
mod parser;
mod price;
mod tax;

// models.rs — data types only
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Asset { Btc, Eur, Gbp, Usd, Other(String) }
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct AssetPair { base: Asset, quote: Asset }
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct AssetAmount { amount: Decimal, asset: Asset }  // checked arithmetic
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum TradeSide { Buy, Sell }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CostBasisMethod { WeightedAverage, Fifo }
#[derive(Debug, Clone, Serialize)]
pub struct Trade { date: DateTime<Utc>, spent: AssetAmount, received: AssetAmount, fee: AssetAmount }
#[derive(Debug, Clone, Serialize)]
pub struct BepSnapshot { date, bep: Option<Decimal>, held, invested, proceeds, fees }
#[derive(Debug, Clone, Serialize)]
pub struct Candle { date: NaiveDate, open, high, low, close: Decimal, volume: Decimal, count: u32 }
#[derive(Debug, Clone, Serialize)]
pub struct TradesSummary { total_trades, buys, sells, unknown, date_range, spent, received, fees }
#[derive(Debug, Clone, Serialize)]
pub struct PositionSummary { bep, held, invested, proceeds, fees, buys, sells }
#[derive(Debug, Clone, Serialize)]
pub struct TaxLot { sell_date, btc_amount, sell_price, assigned_cost, realized_pnl, lots_consumed }
#[derive(Debug, Clone, Serialize)]
pub struct LotMatch { buy_date, btc_amount, cost_per_btc }

// errors.rs — all error and result types
#[derive(Debug, Error)]
pub enum ParseError { Csv(#[from] csv::Error), Io(#[from] std::io::Error), InvalidRow { line, message } }
#[derive(Debug, Error)]
pub enum DbError { Sql(#[from] rusqlite::Error) }
#[derive(Debug, Error)]
pub enum PriceError { Io(#[from] std::io::Error), Csv(#[from] csv::Error), Http(#[from] reqwest::Error), InvalidResponse(String), UnsupportedCurrency(String) }
#[derive(Debug, Error)]
pub struct AssetMismatch { expected: Asset, got: Asset }
#[derive(Debug, Error)]
pub enum EngineError { AssetMismatch(#[from] AssetMismatch) }
#[derive(Debug, Error)]
pub enum CoreError { Parse(#[from] ParseError), Db(#[from] DbError), Price(#[from] PriceError), Engine(#[from] EngineError) }

pub type ParseResult<T> = Result<T, ParseError>;
pub type DbResult<T> = Result<T, DbError>;
pub type PriceResult<T> = Result<T, PriceError>;
pub type EngineResult<T> = Result<T, EngineError>;
pub type CoreResult<T> = Result<T, CoreError>;

// context.rs
pub struct Context { conn: Connection, quote: Asset }
impl Context {
    pub fn open(path: &Path, quote: Asset) -> CoreResult<Self>;
    pub fn quote(&self) -> &Asset;
}

// parser.rs
struct LedgerEntry { txid, refid, time, type_, subtype, aclass, subclass, asset, wallet, amount, fee, balance }
pub(crate) fn parse_kraken_csv(path: &Path) -> ParseResult<Vec<Trade>>;

// engine.rs
pub(crate) fn bep_snaps(pair: &AssetPair, trades: &[Trade]) -> EngineResult<BTreeMap<NaiveDate, BepSnapshot>>;
pub(crate) fn position_summary(pair: &AssetPair, trades: &[Trade]) -> EngineResult<PositionSummary>;
pub(crate) fn trades_summary(pair: &AssetPair, trades: &[Trade]) -> EngineResult<TradesSummary>;

// tax.rs
pub(crate) fn compute_tax_lots(trades: &[Trade], method: CostBasisMethod) -> Vec<TaxLot>;

// price.rs
pub(crate) fn load_bundled_prices(dir: &Path, quote: &Asset) -> PriceResult<Vec<Candle>>;
pub(crate) fn fetch_ohlc(quote: &Asset, since: NaiveDate) -> PriceResult<Vec<Candle>>;
pub(crate) fn fetch_ticker(quote: &Asset) -> PriceResult<Decimal>;

// db.rs
pub(crate) fn open(path: &Path) -> DbResult<Connection>;
pub(crate) fn save_trades(conn: &Connection, trades: &[Trade]) -> DbResult<()>;
pub(crate) fn load_trades(conn: &Connection) -> DbResult<Vec<Trade>>;
pub(crate) fn save_candles(conn: &Connection, quote: &Asset, candles: &[Candle]) -> DbResult<()>;
pub(crate) fn load_candles(conn: &Connection, quote: &Asset) -> DbResult<Vec<Candle>>;

// api.rs — orchestration, most functions take &Context
pub fn preview_import(quote: &Asset, path: &Path) -> CoreResult<TradesSummary>;
pub fn confirm_import(ctx: &Context, path: &Path) -> CoreResult<TradesSummary>;
pub fn candles(ctx: &Context, prices_dir: &Path) -> CoreResult<Vec<Candle>>;
pub fn bep_snaps(ctx: &Context) -> CoreResult<BTreeMap<NaiveDate, BepSnapshot>>;
pub fn position_summary(ctx: &Context) -> CoreResult<PositionSummary>;
pub fn trades(ctx: &Context) -> CoreResult<Vec<Trade>>;
```

From outside the crate, the full public surface is:

```rust
app_core::context::Context::open(...)
app_core::api::preview_import(quote, path)
app_core::api::confirm_import(ctx, path)
app_core::api::candles(ctx, prices_dir)
app_core::api::bep_snaps(...)
app_core::api::position_summary(...)
app_core::api::trades(...)
app_core::models::Trade
app_core::models::Candle
app_core::models::BepSnapshot
app_core::errors::CoreError
app_core::errors::CoreResult
// ...
```

### Tauri Shell

Every Tauri command is a one-liner: extract `State<Context>`, call `app_core::api`, map the error.

```rust
#[tauri::command]
fn preview_import(path: PathBuf, ctx: State<Context>) -> Result<TradesSummary, AppError> {
    Ok(app_core::api::preview_import(ctx.quote(), &path)?)
}

#[tauri::command]
fn confirm_import(path: PathBuf, ctx: State<Context>) -> Result<TradesSummary, AppError> {
    Ok(app_core::api::confirm_import(&ctx, &path)?)
}
```

Live price polling is a Tauri background task, but the HTTP call itself lives in `app-core` (`price::fetch_ticker`). Tauri handles only scheduling and event emission:

```rust
// Tauri background task every ~30s
let price = app_core::price::fetch_ticker(&quote)?;  // core does the HTTP call
app.emit("price_tick", PriceTick { price, quote });   // Tauri emits the event
```

```typescript
// Frontend listens
listen("price_tick", (event) => {
    currentPrice = event.payload.price;
    unrealizedPnl = (currentPrice - stats.current_bep) * stats.btc_held;
});
```

The Tauri crate has no business logic — only command wiring, state management, and event scheduling.

## Core Principle: Dashboard vs Tax Layer

All user-facing data — BEP, BEP chart, unrealized P&L, realized P&L, portfolio stats — always uses weighted average. BTC is fungible; "which coin you sold" is a tax accounting fiction, not a financial reality. This matches industry standard (Interactive Brokers, Koinly, CoinTracker all show portfolio P&L as weighted average).

**BEP formula**: (total fiat spent − total fiat received) / BTC currently held. Pure cash-flow math, independent of cost basis method.

The cost basis method setting (FIFO, etc.) only affects a dedicated tax reporting view with per-sell lot matching and gain/loss breakdowns for tax compliance. It never touches the dashboard.

## Features

### 1. Kraken CSV Import

Drag & drop a Kraken ledger CSV export. The Rust backend parses it into a normalized trade model, extracting buys, sells, and fees with full fiat price information. This is the sole data source for v1 — assumes Kraken is the user's only exchange.

Implemented as a pluggable parser module so other exchange formats (Bitstamp, Coinbase, etc.) can be added later without touching the core logic.

### 2. BEP Computation Engine

Core calculation run in Rust. Processes all trades chronologically:

- **Buy**: increases total BTC held and total fiat spent. BEP = total fiat spent / total BTC held.
- **Sell**: reduces BTC held, crystallizes realized P&L at the sell price vs current BEP. Total fiat spent adjusts proportionally.
- **Fees**: added to total fiat spent (increase your BEP).

Produces a time series of BEP snapshots stored in SQLite, one per trade. This series is what gets plotted against price history.

### 3. Interactive Chart

The main view. Two overlaid line series on a lightweight-charts canvas:

- **BTC price line** (historical daily close from Kraken OHLC API + live ticker)
- **BEP line** (stepped, changes only on buy/sell events)
- **Buy/sell markers** on the timeline using series markers API

Crosshair hover shows: date, BTC price, your BEP, and P&L at that point in time. Dark themed to match financial app conventions.

### 4. Status Dashboard

A header/sidebar panel showing current state at a glance:

- Current BTC price (live from Kraken public ticker, in selected currency)
- Your current BEP
- Unrealized P&L in fiat and %
- Distance from BEP as % ("12.4% above your break-even")
- Total BTC held
- Total fiat invested

Updated in real-time as price ticks come in. Implemented as a Rust background task that polls Kraken's public ticker every ~30s and pushes updates to the frontend via Tauri events.

### 5. Trade Table

Sortable/filterable list of all imported trades showing: date, type (buy/sell), amount BTC, price, fee, total cost, and running BEP after each trade. Each sell row shows its realized gain/loss (weighted average), color-coded green/red.

Export to CSV for external use.

### 6. Historical Price Data

The app ships with bundled CSVs of daily BTC OHLC candles for EUR, GBP, and USD from 2013 to the release date (~4,000 rows each, negligible file size). Stored as full OHLC candles (`Candle` type: date, open, high, low, close, volume, count).

**Data flow**: On first call to `api::candles()`, bundled CSVs are parsed and saved to SQLite. If the most recent candle is older than today, the gap is filled via Kraken's public OHLC API (`/0/public/OHLC?pair=XBTEUR&interval=1440&since={timestamp}`). The DB uses `UNIQUE(quote, date)` with upsert, so overlapping candles from gap-fills are safely overwritten. Subsequent calls read directly from DB.

No API keys required. Kraken's public OHLC endpoint (unauthenticated) is the sole external dependency for historical data. Live price uses the separate Ticker endpoint. The bundled CSVs are updated with each app release, so the app renders a chart instantly on first launch.

### 7. Realized Gains

Derived automatically from imported trades.

- **Dashboard card**: total realized P&L (weighted average) across all sells, alongside unrealized P&L.
- **Per-trade P&L**: each sell in the trade table shows its realized gain/loss, color-coded green/red.
- **Time filter**: filter by year or custom range to see realized gains over any period.
- **Tax view**: a separate per-sell breakdown using the selected cost basis method (FIFO, etc.) with assigned lot matching and gain/loss. This is the layer users consult for tax reporting, not the main dashboard.

### 8. Currency & Cost Basis Settings

The user selects their base currency and cost basis method on first launch (changeable in settings). The currency drives all displayed values and chart labels. The cost basis method only affects the tax view (see core principle above).

**Supported currencies**: EUR, GBP, USD. The app bundles historical daily BTC close prices for all three pairs. Live price fetched from Kraken's public Ticker for the selected pair.

**Supported cost basis methods**:

| Method | Used by | How it works |
|--------|---------|--------------|
| Weighted Average | Spain, France, Netherlands, UK (Section 104) | All holdings share one average cost = total spent / total held. Matches the BEP. |
| FIFO | US (default), Germany, Italy | Sells consume the oldest unsold lots first. |

The trade data is the same regardless of method — only the tax view changes.

### 9. Import Validation Report

After CSV import, the app shows a summary before committing anything:

- **Counts**: rows parsed, rows skipped (non-BTC assets, deposits, withdrawals), trades imported (buys and sells separately)
- **Date range**: first and last trade date ("Jan 2019 → Feb 2026")
- **Totals**: total BTC bought, total BTC sold, total fiat spent, total fiat received, total fees paid
- **Warnings**: duplicate rows detected (already imported), unrecognized row types, rows that failed to parse (with line numbers)
- **Net position**: "You hold X BTC across Y buy trades" as a sanity check against their Kraken balance

A single card with these numbers and a confirmation action. No charts, no BEP yet — just proof the data landed correctly. Builds trust and catches bad imports early (wrong CSV, duplicate imports, missing rows).

### 10. DCA Zones Overlay

A toggle on the main chart that highlights buy periods relative to the current BEP. Green zones where you bought below your BEP (buys that lowered your average), red zones where you bought above (buys that pushed it up). Instantly shows whether your DCA strategy is working over time.

### 11. Accumulation Chart

A separate area chart showing total BTC held over time. Not about price — purely about your stack growing. Steps up on buys, steps down on sells. Can toggle between BTC and sats on the y-axis.

### 12. Holder Stats

Fun, motivational metrics that make the app worth opening:

- **Diamond hands score**: how long you've held without selling, or what % of your total BTC you've never sold.
- **Sats per day**: average accumulation rate across your entire history. "You're stacking 12,450 sats/day on average."

### 13. Privacy & Trust

**Privacy visibility in the UX**:

- **Network indicator**: a status bar icon showing when the app makes network requests (price fetching only) and when it's fully offline. No hidden telemetry.
- **Data location badge**: in settings, show the exact path where all data lives (e.g. `~/.betc/data.db`). One click to open the folder.
- **First-launch card**: brief onboarding notice — "betc runs entirely on your machine. Your transaction data never leaves this device."
- **No accounts, no login, no cloud**: implicit but made explicit in the UX. Zero sign-up flows.

**Binary integrity**:

- **SHA-256 checksums**: published on the GitHub release page and a second channel (website, Nostr) so compromising one isn't enough.
- **GPG-signed checksums**: checksum file signed with maintainer's GPG key for full verification chain.
- **Build-from-source instructions**: clear docs for users who want to compile themselves (Rust + Node, standard Tauri build).
- **Unsigned binaries**: no paid code signing. macOS users right-click → Open on first launch, Windows users dismiss SmartScreen. The source code and checksums are the trust layer, not a certificate authority.
- **Distribution**: GitHub Releases only. Direct download with checksums and GPG sigs alongside each release.

## Build Order

Four phases. Each builds on the previous and produces a testable milestone.

**Phase 1 — Data in** (features 1, 9, 6): Kraken CSV parser, import validation, historical price loading into SQLite. Testable in Rust with no UI — unit tests proving correct parsing and price data.

**Phase 2 — Math works** (features 2, 8, 7): BEP engine, currency & cost basis settings, realized gains computation (weighted average for dashboard, lot-based for tax view). Still no UI needed — Rust tests with known trade sets asserting correct BEP and P&L.

**Phase 3 — MVP ships** (features 3, 4, 5): Interactive chart, status dashboard, trade table. First time the frontend matters. After this phase, the app is usable end-to-end.

**Phase 4 — Polish** (features 10, 11, 12, 13): DCA zones, accumulation chart, holder stats, privacy UX. The app already works without any of these.

## UI & UX Layout

How features map to what the user actually sees.

### App Shell

```
┌──────┬─────────────────────────────────────────────┐
│      │  ┌─ Status Bar ───────────────────────────┐  │
│  N   │  │ BTC €97,420  ▲2.1%  │  🟢 Online      │  │
│  A   │  └────────────────────────────────────────┘  │
│  V   │                                              │
│      │  ┌─ Main Content ─────────────────────────┐  │
│  B   │  │                                        │  │
│  A   │  │   (swaps based on active nav item)     │  │
│  R   │  │                                        │  │
│      │  │                                        │  │
│      │  └────────────────────────────────────────┘  │
└──────┴──────────────────────────────────────────────┘
```

**Nav bar** (left, vertical, icon-only, Discord-style): switches between views. No URLs, just component swapping via Svelte state.

**Status bar** (top, persistent across all views): live BTC price, change %, network indicator. Always visible — the user never loses sight of current price.

### Views (nav bar items)

**Overview** — the default view, where the user spends most time. Features 3, 4, 7, 10, 12.

- Top row: status dashboard cards (feature 4) — current BEP, unrealized P&L in fiat and %, distance from BEP, total BTC held, total fiat invested. Realized P&L card here too (feature 7).
- Main area: interactive chart (feature 3) with BTC price line, BEP line, buy/sell markers. DCA zones overlay as a toggle on the chart toolbar (feature 10).
- Below chart or in a collapsible panel: holder stats (feature 12) — diamond hands score, sats per day. Lightweight, motivational, not competing for attention with the chart.

**Trades** — feature 5. Full-screen sortable/filterable table. Each row shows running BEP and per-trade realized P&L (feature 7, color-coded). Export button in toolbar.

**Accumulation** — feature 11. Full-screen area chart showing BTC held over time. BTC/sats toggle on y-axis. Simple, focused view with no other elements competing.

**Settings** — feature 8, part of 13. Currency selector, cost basis method, data location badge with one-click to open folder. Minimal.

### Modals & Flows (not in nav)

**Import flow** (features 1, 9): triggered by a prominent button in the overview (empty state) or a smaller import action in the nav/toolbar. Opens a modal sequence: drag & drop zone → parsing progress → validation report → confirm. Not a separate view — it's a task the user completes and returns to overview.

**First-launch onboarding** (features 8, 13): shown once on first open. A single card or short stepper: privacy notice ("everything stays local"), currency picker, cost basis method. Three steps max, then straight to the import flow.

### Features with no dedicated view

These features are pure backend or are surfaced within other views:

- **Feature 2 (BEP engine)**: computation layer, no UI of its own.
- **Feature 6 (Historical price data)**: data pipeline, feeds the chart.
- **Feature 7 (Realized gains)**: displayed in overview dashboard cards and trade table rows.
- **Feature 13 (Privacy & trust)**: distributed — network indicator in status bar, data location in settings, first-launch card in onboarding. Not a standalone view.

## Visual Identity

Dark mode only. No toggle — financial apps live in dark mode, and a single theme means half the design surface to maintain.

**Mood**: Calm, trustworthy, personal. This is a clarity tool you open to check on your position, not a trading terminal that screams urgency. Closer to a banking app than a crypto exchange.

**Color palette**:

| Role | Color | Value | Usage |
|------|-------|-------|-------|
| Background | Zinc 950 | `#09090b` | App background, cards |
| Surface | Zinc 900 | `#18181b` | Elevated panels, nav bar, status bar |
| Border | Zinc 800 | `#27272a` | Card borders, dividers, subtle separators |
| Muted text | Zinc 400 | `#a1a1aa` | Secondary labels, timestamps, captions |
| Primary text | Zinc 50 | `#fafafa` | Headings, values, primary content |
| Accent | Amber 500 | `#f59e0b` | Active nav item, selected state, primary buttons. Subtle Bitcoin nod without being on-the-nose orange. |
| Gain | Emerald 400 | `#34d399` | Positive P&L, green DCA zones, "above BEP" |
| Loss | Red 400 | `#f87171` | Negative P&L, red DCA zones, "below BEP" |
| BTC price line | Zinc 300 | `#d4d4d8` | Chart: BTC price series. Neutral, it's the reference. |
| BEP line | Amber 400 | `#fbbf24` | Chart: your break-even price. The hero — warm, distinct, immediately identifiable. |
| Buy marker | Emerald 500 | `#10b981` | Chart: buy event dots |
| Sell marker | Red 500 | `#ef4444` | Chart: sell event dots |

The accent is amber, not Bitcoin orange (`#F7931A`). Close enough to evoke BTC, different enough to not look like a Bitcoin.org clone. The BEP line uses the same amber family — it's *your* line, the most important thing on screen.

**Typography**:

- **UI text**: System font stack (`-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif`). No custom font loading — instant render, native feel on every OS.
- **Numbers**: Tabular lining figures. All monetary values, percentages, and quantities should use `font-variant-numeric: tabular-nums` so columns align and values don't jitter when they update. Apply via Tailwind: `tabular-nums` class.
- **Monospace**: Only for raw data display (transaction IDs, file paths). System mono (`ui-monospace, "SF Mono", monospace`).
- **Scale**: shadcn defaults. Don't fight the system — `text-sm` for most content, `text-lg` or `text-xl` for dashboard hero numbers (current price, BEP, P&L).

**Spacing & density**:

Finance apps are information-dense by nature. Don't fight that — lean into it with tight but consistent spacing:

- Dashboard cards: compact, no excessive padding. Show 4–6 cards in a single row without scrolling.
- Trade table: dense rows (32–36px height), readable but not airy. Users scanning hundreds of trades don't want whitespace.
- Chart: maximum available space. The chart should fill the remaining viewport after dashboard cards — it's the primary content.
- Nav bar: 56–64px wide, icon-only, no labels. Tooltip on hover for discoverability.

**Borders & elevation**:

No drop shadows. Depth comes from background color shifts (Zinc 950 → 900 → 800) and 1px borders. Cards use `border border-zinc-800` on a `bg-zinc-900` surface against the `bg-zinc-950` app background. Subtle, no floating paper metaphors.

**Interaction states**:

- Hover: lighten one step (Zinc 800 → 700 for surfaces, amber glow for accent elements)
- Active/selected: amber accent (nav icons, selected tab)
- Focus: amber ring (`ring-amber-500/50`), visible but not harsh
- Disabled: Zinc 600 text, no other change

**Charts (lightweight-charts specific)**:

```
Background:      transparent (inherits app background)
Grid lines:      Zinc 800, horizontal only, no vertical
Crosshair:       Zinc 500, dashed
Price scale text: Zinc 400
Time scale text:  Zinc 400
```

The chart should feel embedded in the UI, not like a widget dropped in from a different app. Transparent background and matching grid colors achieve this.

**Logo**:

Open decision. Direction: minimal, geometric, monochrome in the nav bar (Zinc 400 → Zinc 50 when active). Could work as just a stylized lettermark or an abstract chart-line icon. Amber accent version for loading screen and about page. No need for it to literally depict Bitcoin — the app name and context do that work.

**App Name**:

Open decision. Candidates: **stak**, **basis**, **proof**, **brek**, **satx**, **betc**, **held**. Direction: short (4-5 letters), punchy, slightly abbreviated or misspelled. Avoids "hodl" cringe. Fits the 2025/2026 Bitcoin culture shift toward stewardship and conviction over speculation.

## Deferred

- **Sell simulator**: "what if I sell X BTC at today's price?" — shows weighted average P&L as hero number with tax-method gain as secondary line. Useful but not essential for MVP; the user can do this mental math from the dashboard numbers.
- **Sparrow wallet CSV import**: contains on-chain tx data (txid, date, BTC amount, fee) but no fiat prices. Doesn't help with BEP since withdrawals from Kraken to cold storage are just internal transfers. Could be useful later for reconciliation or tracking BTC from external sources — but those would need manual fiat value assignment.
- **UK 30-day bed & breakfast rule**: if you rebuy within 30 days of selling, the rebuy cost overrides the Section 104 pool for that portion. Edge case, can add later.
- **LIFO/HIFO cost basis methods**: additional methods for broader jurisdiction support.
- **Reproducible builds**: byte-for-byte identical binaries from source. Achievable with Rust + Tauri but requires work on eliminating timestamps and linker non-determinism.
- **Tax summary export**: generated reports for specific jurisdictions (Spain Modelo 720/100, UK CGT, US Form 8949). Would make the app useful for tax season but changes its scope significantly.
- **SLSA build provenance**: GitHub Actions attestation proving which commit produced which binary. Modern supply chain verification.
- **Package manager distribution**: Homebrew Cask (macOS), Scoop/winget (Windows), Flathub (Linux) as convenience install channels.

## Open Decisions

- [ ] App name (candidates and direction defined in Visual Identity)
- [ ] Logo design (direction defined in Visual Identity: minimal, geometric, monochrome nav icon with amber accent variant)
