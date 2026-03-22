# Kraken Ledger CSV Format Reference

_Compiled from official Kraken API documentation, support articles, and community sources. Last updated: 2026-03-22._

This document describes the CSV format exported by Kraken's **Ledger** export (History → Export → Ledgers). It serves as the reference for building test fixtures and extending Satfolio's Kraken parser.

---

## CSV Columns

The ledger CSV has 12 columns with the following fixed header:

```
txid,refid,time,type,subtype,aclass,subclass,asset,wallet,amount,fee,balance
```

| Column     | Type    | Description                                                                        |
| ---------- | ------- | ---------------------------------------------------------------------------------- |
| `txid`     | string  | Unique ledger entry identifier                                                     |
| `refid`    | string  | Reference ID linking related entries (e.g., both sides of a trade share a `refid`) |
| `time`     | string  | Timestamp in `YYYY-MM-DD HH:MM:SS` format (UTC)                                    |
| `type`     | string  | Broad event classification (see [Type Values](#type-values))                       |
| `subtype`  | string  | Specific classification, often empty (see [Subtype Values](#subtype-values))       |
| `aclass`   | string  | Asset class — always `"currency"`                                                  |
| `subclass` | string  | Asset subclass — `"fiat"` or `"crypto"`                                            |
| `asset`    | string  | Asset code using Kraken's naming (see [Asset Naming](#asset-naming))               |
| `wallet`   | string  | Wallet identifier (see [Wallet Types](#wallet-types))                              |
| `amount`   | decimal | Signed change amount — negative = outflow, positive = inflow                       |
| `fee`      | decimal | Fee amount denominated in the entry's asset                                        |
| `balance`  | decimal | Account balance in this asset after the entry                                      |

---

## Type Values

16 possible values for the `type` field:

| Type          | Description                                              | Relevant to BTC holders?                                 |
| ------------- | -------------------------------------------------------- | -------------------------------------------------------- |
| `trade`       | Exchange of one asset for another (paired entries)       | **Yes** — core trade parsing                             |
| `spend`       | Fiat or crypto spent, typically paired with `receive`    | **Yes** — alternative trade format ("Buy Crypto" button) |
| `receive`     | Crypto received, typically paired with `spend`           | **Yes** — alternative trade format                       |
| `staking`     | Staking/earn reward received                             | **Yes** — BTC staking rewards                            |
| `deposit`     | Funds deposited into account                             | No — not a trade                                         |
| `withdrawal`  | Funds withdrawn from account                             | No — not a trade                                         |
| `transfer`    | Movement between wallet types (see subtypes)             | Partial — staking allocation/deallocation                |
| `adjustment`  | Account adjustment (delisted token conversion, airdrops) | Rare — forced conversions                                |
| `margin`      | Margin position opening                                  | No — out of scope                                        |
| `rollover`    | Fee for maintaining open margin position                 | No — out of scope                                        |
| `settled`     | Margin position settlement/closure                       | No — out of scope                                        |
| `credit`      | Account credit                                           | Rare                                                     |
| `sale`        | Direct asset sale                                        | Rare                                                     |
| `conversion`  | Currency conversion                                      | Rare                                                     |
| `dividend`    | Dividend payment                                         | Rare                                                     |
| `reward`      | Reward (documented but rarely seen in practice)          | Rare                                                     |
| `creator_fee` | NFT creator fee                                          | No                                                       |

> **Note (from Rotki, 2025-12-10):** Types `reward`, `conversion`, `credit`, `dividend`, `sale`, `nfttrade`, `nftcreatorfee`, `nftrebate`, `custodytransfer` exist in the schema but have limited real-world observations. They may need further adjustment.

---

## Subtype Values

6 possible values for the `subtype` field (most entries have an empty subtype):

| Subtype           | Used with type | Description                                              |
| ----------------- | -------------- | -------------------------------------------------------- |
| `spottostaking`   | `transfer`     | Moving asset from spot wallet to staking                 |
| `stakingfromspot` | `transfer`     | Staking wallet receives from spot (counterpart of above) |
| `stakingtospot`   | `transfer`     | Moving asset from staking back to spot                   |
| `spotfromstaking` | `transfer`     | Spot wallet receives from staking (counterpart of above) |
| `spottofutures`   | `transfer`     | Moving asset from spot to futures wallet                 |
| `spotfromfutures` | `transfer`     | Moving asset from futures to spot wallet                 |

The `trade` type also uses `subtype` with values like `"tradespot"`, though this is not documented in the official enum.

---

## Entry Pairing Patterns

Kraken's ledger records each side of a transaction as a separate entry. Related entries share the same `refid`.

### Trade (type=`trade`)

Two entries with the same `refid`:

- One with negative `amount` (asset sold)
- One with positive `amount` (asset bought)
- Fee appears on one side (typically the sell side, but can be on either)

```csv
"LG001","REF001","2025-02-14 22:18:09","trade","tradespot","currency","fiat","EUR","spot / main",-187.2514,0.7490,193.9828
"LG002","REF001","2025-02-14 22:18:09","trade","tradespot","currency","crypto","BTC","spot / main",0.0020104289,0,0.0030521628
```

### Spend/Receive (type=`spend` + type=`receive`)

Two entries with the same `refid` — alternative trade format used by Kraken's "Buy Crypto" button:

- `spend` entry: negative amount (fiat spent)
- `receive` entry: positive amount (crypto received)

```csv
"LG003","REF002","2025-03-10 15:00:00","spend","","currency","fiat","EUR","spot / main",-100.00,0.50,50.00
"LG004","REF002","2025-03-10 15:00:00","receive","","currency","crypto","BTC","spot / main",0.0011,0,0.0041
```

### Staking Reward (type=`staking`)

**Single entry** — no matching pair. Positive amount representing the reward received:

```csv
"LG005","","2025-04-01 00:00:00","staking","","currency","crypto","BTC.M","spot / main",0.00000123,0,0.00300123
```

### Staking Allocation / Deallocation

4 entries for a stake operation (2 pairs by `refid`):

1. `withdrawal` from spot (negative amount)
2. `deposit` to staking variant (positive amount, asset suffix `.S` or `.M`)
3. `transfer` with subtype `spottostaking`
4. `transfer` with subtype `stakingfromspot`

Unstaking reverses the pattern with `stakingtospot` / `spotfromstaking`.

### Deposit / Withdrawal

Single entry:

```csv
"LG006","REF003","2025-02-14 20:32:16","deposit","","currency","fiat","EUR","spot / main",381.9821,0,381.9832
```

### Failed / Cancelled Event

Two entries with the **same type and subtype**, whose amounts sum to zero. Rotki detects these and skips them:

```csv
"LG007","REF004","2025-05-01 10:00:00","withdrawal","","currency","crypto","BTC","spot / main",-0.01,0.0001,0.99
"LG008","REF004","2025-05-01 10:05:00","withdrawal","","currency","crypto","BTC","spot / main",0.01,0.0001,1.00
```

### Adjustment

Paired entries, type=`adjustment` — used for forced conversion of delisted assets, airdrops, or fork credits. One negative (asset removed), one positive (asset credited):

```csv
"LG009","REF005","2025-06-01 00:00:00","adjustment","","currency","crypto","XXBT","spot / main",-0.001,0,0.999
"LG010","REF005","2025-06-01 00:00:00","adjustment","","currency","fiat","ZUSD","spot / main",50.00,0,50.00
```

---

## Asset Naming

Kraken uses its own asset codes, which differ from standard ticker symbols:

| Kraken code     | Standard | Notes                                       |
| --------------- | -------- | ------------------------------------------- |
| `XXBT`, `XBT`   | BTC      | Bitcoin                                     |
| `XETH`          | ETH      | Ethereum                                    |
| `ZEUR`          | EUR      | Euro (Z-prefix for fiat)                    |
| `ZUSD`          | USD      | US Dollar                                   |
| `ZGBP`          | GBP      | British Pound                               |
| `KFEE`, `ZKFEE` | —        | Fee credit tokens (**zero monetary value**) |
| `ETH2.S`        | —        | Staked ETH variant                          |
| `BTC.M`         | —        | BTC in earn/staking product                 |

The `Z` prefix is used for fiat currencies, `X` prefix for some crypto assets. Not all assets use these prefixes consistently — newer listings may use standard tickers directly.

---

## Wallet Types

| Wallet            | Description                       |
| ----------------- | --------------------------------- |
| `spot / main`     | Primary spot trading wallet       |
| `earn / bonded`   | Earn product with lockup period   |
| `earn / flexible` | Earn product without lockup       |
| `earn / liquid`   | Kraken rewards program            |
| `earn / locked`   | Earn product with variable lockup |

---

## KFEE Token

Kraken's KFEE is a fee credit token used to discount trading fees. Important behaviors:

- KFEE has **no market price** — it cannot be traded or valued
- Fees denominated in KFEE should be treated as **zero monetary value**
- Rotki explicitly filters KFEE: `if asset != A_KFEE` before creating non-fee events

---

## Sources

- **Kraken WebSocket v2 Balances API** — type/subtype/category enumerations, wallet types
  https://docs.kraken.com/api/docs/websocket-v2/balances/

- **Kraken Support: How to interpret Ledger history fields** — column descriptions, field semantics
  https://support.kraken.com/articles/360001169383-how-to-interpret-ledger-history-fields

- **Kraken REST API: Get Ledgers** — ledger query endpoint documentation
  https://docs.kraken.com/api/docs/rest-api/get-ledgers/

- **CoinTaxman Issue #97** — community-sourced catalog of real-world Kraken CSV entry types with examples
  https://github.com/provinzio/CoinTaxman/issues/97

- **Rotki `kraken_ledger_entry_type_to_ours()`** — type mapping function (AGPL-3.0), includes notes on observed vs unobserved types
  https://github.com/rotki/rotki/blob/develop/rotkehlchen/exchanges/kraken.py
