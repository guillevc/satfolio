# Kraken Ledger CSV Format Reference

_Based on [Kraken's official support article](https://support.kraken.com/articles/360001169383-how-to-interpret-ledger-history-fields) and verified against real CSV exports. Last updated: 2026-03-22._

This document describes the CSV format exported by Kraken's **Ledger** export (History → Export → Ledgers). It serves as the reference for building test fixtures and extending Satfolio's Kraken parser.

---

## CSV Columns

The ledger CSV has 12 columns with the following fixed header:

```
txid,refid,time,type,subtype,aclass,subclass,asset,wallet,amount,fee,balance
```

| Column     | Type    | Description (from Kraken docs)                                                                                  |
| ---------- | ------- | --------------------------------------------------------------------------------------------------------------- |
| `txid`     | string  | Transaction ID. Matches the Ledger ID on Kraken.com → History → Ledger.                                         |
| `refid`    | string  | Reference ID linking related entries (e.g., both sides of a trade share a `refid`)                              |
| `time`     | string  | Date & time in UTC. Format: `YYYY-MM-DD HH:MM:SS`                                                               |
| `type`     | string  | Event type (see [Type Values](#type-values))                                                                    |
| `subtype`  | string  | Specific classification, often empty (see [Subtype Values](#subtype-values))                                    |
| `aclass`   | string  | Asset class — always `"currency"`. Not useful.                                                                  |
| `subclass` | string  | Asset subclass — `"fiat"` or `"crypto"`. Not in official docs but present in exports.                           |
| `asset`    | string  | The asset for this entry (see [Asset Naming](#asset-naming)). Amount, fee, and balance are denominated in this. |
| `wallet`   | string  | Wallet identifier (see [Wallet Types](#wallet-types))                                                           |
| `amount`   | decimal | Amount debited (−) or credited (+) to that asset's balance.                                                     |
| `fee`      | decimal | Fee paid to Kraken (if any) in the asset.                                                                       |
| `balance`  | decimal | New asset balance after debiting/crediting amount and debiting fee. `balance = old_balance +/- amount - fee`    |

> **Note:** Each ledger entry focuses on a change to a particular asset's balance. Two different assets are never mixed in a single entry.

---

## Type Values

From Kraken's [official support article](https://support.kraken.com/articles/360001169383-how-to-interpret-ledger-history-fields):

| Type           | Description (Kraken docs, verbatim or paraphrased)                                                                                                                                        | Relevant to BTC holders?                                 |
| -------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------- |
| `trade`        | Non-margin exchange of one currency for another.                                                                                                                                          | **Yes** — core trade parsing                             |
| `earn`         | Relates to all on-chain staking and Opt-In Rewards activities, including allocation, deallocation, rewards, and other transactions. Encompasses both types of staking under one category. | **Yes** — BTC earn/staking rewards (with `subtype`)      |
| `spend`        | Amount of asset debited, for transactions via the Buy Crypto button or Kraken app.                                                                                                        | **Yes** — alternative trade format (paired with receive) |
| `receive`      | Amount of asset credited, for transactions via the Buy Crypto button or Kraken app.                                                                                                       | **Yes** — alternative trade format (paired with spend)   |
| `staking`      | Primarily used for staking rewards.                                                                                                                                                       | **Yes** — legacy staking rewards                         |
| `deposit`      | Deposit of funds, including KFEE credits and Futures wallet transfers.                                                                                                                    | No — not a trade                                         |
| `withdrawal`   | Withdrawal of funds outside of Kraken account.                                                                                                                                            | No — not a trade                                         |
| `transfer`     | Credit of airdrops/forks, OTC transfers, Futures wallet transfers. Also staking movements when paired with staking-related subtypes.                                                      | Partial — staking allocation/deallocation                |
| `adjustment`   | Conversion of one currency to another outside of trading (e.g. ICN-to-ETH after delisting).                                                                                               | Rare — forced conversions                                |
| `margin trade` | Profits/loss for a margin trade.                                                                                                                                                          | No — out of scope                                        |
| `rollover`     | Charge for a margin trade.                                                                                                                                                                | No — out of scope                                        |
| `settled`      | Settling of a margin position on spot.                                                                                                                                                    | No — out of scope                                        |
| `sale`         | Filter-only — **not shown in CSV exports**. Brings up spend/receive entries from the Kraken app.                                                                                          | N/A — not in CSV                                         |
| `invite bonus` | Credit of rewards from Kraken app referral program.                                                                                                                                       | Rare                                                     |

---

## Subtype Values

From Kraken's [official support article](https://support.kraken.com/articles/360001169383-how-to-interpret-ledger-history-fields):

| Subtype           | Used with type | Description (Kraken docs)                                                                          |
| ----------------- | -------------- | -------------------------------------------------------------------------------------------------- |
| `allocation`      | `earn`         | Allocation of assets — transfer from Spot balance to Earn balance.                                 |
| `deallocation`    | `earn`         | Transfer of allocated assets from Earn balance back to Spot balance.                               |
| `autoallocate`    | `earn`         | Automatic allocation to on-chain Staking or Opt-In Rewards through Kraken Rewards.                 |
| `reward`          | `earn`         | Payouts from on-chain Staking or Opt-In Rewards. Typically issued weekly or bi-weekly.             |
| `migration`       | `earn`         | Transfer of staked balance from legacy staking to new staking infrastructure. No portfolio impact. |
| `spottostaking`   | `transfer`     | Allocation of base assets for staking (−) — transfer out of spot account.                          |
| `stakingfromspot` | `transfer`     | Allocation credited as staked assets (+) — transfer into staking balance.                          |
| `stakingtospot`   | `transfer`     | Deallocation of staked assets (−) — transfer out of staking balance.                               |
| `spotfromstaking` | `transfer`     | Deallocation credited back as base assets (+) — transfer into spot account.                        |
| `spottofutures`   | `transfer`     | Transfer from spot to futures.                                                                     |
| `spotfromfutures` | `transfer`     | Transfer from futures to spot.                                                                     |

The `trade` type also uses subtypes like `"tradespot"` and `"autoallocation"` in real exports, though these are not listed in the official docs.

---

## Entry Pairing Patterns

Kraken's ledger records each side of a transaction as a separate entry. Related entries share the same `refid`. Examples below are from real CSV exports.

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

Two entries with the same `refid` — used by Kraken's "Buy Crypto" button and recurring buys:

```csv
"L7T3BD","IR9UY7S","2025-07-01 04:01:33","spend","","currency","fiat","EUR","spot / main",-28.3660,0.2827,120.8850
"LV3Z0G","IR9UY7S","2025-07-01 04:01:33","receive","","currency","crypto","BTC","spot / main",0.0003102764,0,0.0003102766
```

### Earn Reward (type=`earn`, subtype=`reward`)

**Single entry** — positive amount representing a staking or Opt-In Rewards payout:

```csv
"LGMSOX","5GAS442","2025-05-31 22:25:09","earn","reward","currency","fiat","EUR","earn / flexible",0.0179,0,149.1339
```

### Legacy Staking Reward (type=`staking`)

**Single entry** — the legacy format, "primarily used for staking rewards":

```csv
"LG005","","2025-04-01 00:00:00","staking","","currency","crypto","BTC.M","spot / main",0.00000123,0,0.00300123
```

### Earn Allocation/Deallocation (type=`earn`, subtype=`allocation`/`autoallocation`)

Paired entries moving assets between spot and earn wallets. Not trades — no portfolio impact:

```csv
"L520RN","ZDANTNI","2025-07-04 15:35:39","earn","autoallocation","currency","crypto","BTC","spot / main",-0.0003102764,0,0.0000000002
"LO69WE","ZDANTNI","2025-07-04 15:35:39","earn","autoallocation","currency","crypto","BTC","earn / liquid",0.0003102764,0,0.0003102764
```

### Deposit / Withdrawal

Single entry:

```csv
"LHBRPO","FTMbJmT","2025-02-14 20:32:16","deposit","","currency","fiat","EUR","spot / main",381.9821,0,381.9832
```

---

## Asset Naming

Kraken uses its own asset codes. See [Kraken's asset code explanation](https://support.kraken.com/articles/360001185506-Explanation-of-Asset-Codes).

| Kraken code   | Standard | Notes                                      |
| ------------- | -------- | ------------------------------------------ |
| `XXBT`, `XBT` | BTC      | Bitcoin (X-prefix for crypto)              |
| `XETH`        | ETH      | Ethereum                                   |
| `ZEUR`        | EUR      | Euro (Z-prefix for fiat)                   |
| `ZUSD`        | USD      | US Dollar                                  |
| `ZGBP`        | GBP      | British Pound                              |
| `KFEE`        | —        | Fee credit token (**zero monetary value**) |
| `BTC.M`       | —        | BTC in Opt-In Rewards program              |
| `ETH2.S`      | —        | Staked ETH variant                         |

Newer asset listings may use standard tickers directly (e.g., `BTC` instead of `XXBT`). Both formats appear in real exports.

---

## Wallet Types

Observed in real CSV exports:

| Wallet            | Description                       |
| ----------------- | --------------------------------- |
| `spot / main`     | Primary spot trading wallet       |
| `earn / flexible` | Earn product without lockup       |
| `earn / bonded`   | Earn product with lockup period   |
| `earn / liquid`   | Kraken Rewards program            |
| `earn / locked`   | Earn product with variable lockup |

---

## KFEE Token

From [Kraken's docs](https://support.kraken.com/articles/204799657-What-are-Kraken-fee-credits-KFEE-): KFEE is a fee credit token used to discount trading fees. Deposits of KFEE appear as `type=deposit`.

- KFEE has **no market price** — it cannot be traded or valued
- Fees denominated in KFEE should be treated as **zero monetary value**

---

## Sources

- **Kraken Support: How to interpret Ledger history fields** — official CSV field documentation
  https://support.kraken.com/articles/360001169383-how-to-interpret-ledger-history-fields

- **Kraken Support: Explanation of Asset Codes** — asset naming conventions
  https://support.kraken.com/articles/360001185506-Explanation-of-Asset-Codes

- **Kraken Support: What are Kraken fee credits (KFEE)?** — KFEE token documentation
  https://support.kraken.com/articles/204799657-What-are-Kraken-fee-credits-KFEE-

- **Real Kraken CSV exports** — verified against `fixtures/kraken/sample.csv` in this repository
