# iati-transform

> Transformations and rollups for [IATI](https://iatistandard.org) Activity data.  
> Part of the [`iati-crates`](https://github.com/codywallace/iati-crates) ecosystem.

[![Crates.io](https://img.shields.io/crates/v/iati-transform.svg)](https://crates.io/crates/iati-transform)
[![Docs.rs](https://docs.rs/iati-transform/badge.svg)](https://docs.rs/iati-transform)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](https://github.com/codywallace/iati-crates)

---

## Overview

This crate provides **pure, IO-free transformations** for [`iati-types`](https://crates.io/crates/iati-types) data structures —  
designed for use in CLI tools, web services, and data pipelines.

### Core functions

- Aggregate transactions by **type**, **year**, or **currency**
- Respect **currency fallback** (`transaction.currency` → `activity.default_currency`)
- Simple, functional design — no I/O, no side effects
- Works seamlessly with [`iati-xml`](https://crates.io/crates/iati-xml) for parsed IATI data

---

## Example

```rust
use iati_transform::{aggregate_by_type, aggregate_by_year_and_type, FxCurrency};
use iati_types::{Activity, TxType, Transaction, Money, CurrencyCode};
use chrono::NaiveDate;
use rust_decimal::Decimal;

// Build a minimal IATI Activity
let mut a = Activity::new("ACT-1");
a.default_currency = Some(CurrencyCode::from("USD"));

a.transactions.push(Transaction::new(
    TxType::Disbursement,
    NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
    Money {
        amount: Decimal::new(1000, 2),
        currency: None, // falls back to USD
        value_date: None,
    },
));

a.transactions.push(Transaction::new(
    TxType::Disbursement,
    NaiveDate::from_ymd_opt(2023, 2, 1).unwrap(),
    Money {
        amount: Decimal::new(500, 2),
        currency: Some(CurrencyCode::from("EUR")),
        value_date: None,
    },
));

// Aggregate by type in native currency space (no FX conversions)
let sums = aggregate_by_type(&[a], FxCurrency::Native);
let total_usd = sums.total_for(TxType::Disbursement, &CurrencyCode::from("USD"));
assert!(total_usd.is_some());

// Aggregate by year and transaction type
let year_sums = aggregate_by_year_and_type(&[a], FxCurrency::Native);
assert!(year_sums.total_for(2023, TxType::Disbursement, &CurrencyCode::from("USD")).is_some());
