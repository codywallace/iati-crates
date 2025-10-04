# iati-types

Core strongly typed data models for the [International Aid Transparency Initiative (IATI)](https://iatistandard.org) Activity Standard v2.03.  
This crate provides **IO-free Rust types** representing the spine of an IATI Activity, focusing on transactions, currencies, and identifiers.

---

## Overview

IATI publishes open data on development and humanitarian activities in XML. While powerful, the raw XML can be complex to work with:  
- Nested structures  
- Multiple transaction types  
- Currency variations  
- Inconsistent reporting practices  

`iati-types` provides clean, strongly typed Rust models that can be used as a foundation for:  
- Parsing and serializing IATI XML (via downstream crates like `iati-xml`)  
- Data cleaning and transaction rollups (`iati-transform`)  
- Currency conversion (`iati-fx`)  
- Building CLI or data pipelines (`iati-cli`)  

This crate is deliberately **IO-free**: it defines only the core types. All parsing/serialization, code list lookups, and transformations happen in other crates.

---

## Features

- `Activity` — minimal spine of an IATI activity (identifier, currency, transactions, reporting org, dates).  
- `Transaction` — transaction struct with builder-style methods.  
- `TxType` — enum for IATI transaction-type codelist (codes 1–13 + `Unknown`).  
- `Money` — amount, optional currency, optional value date.  
- `CurrencyCode` — ISO 4217 wrapper, normalized to uppercase.  
- `OrgRef` — lightweight organisation reference (id + name).  

Optional **serde** support (enabled by default) for easy serialization.

---

## Example

```rust
use iati_types::{Activity, Transaction, TxType, Money, CurrencyCode, OrgRef};
use chrono::NaiveDate;
use rust_decimal::Decimal;

fn main() {
    let date = NaiveDate::from_ymd_opt(2023, 5, 1).unwrap();
    let money = Money::new(Decimal::new(5000, 2));

    let tx = Transaction::new(TxType::Disbursement, date, money)
        .with_provider(OrgRef { ref_id: Some("AAA-111".into()), name: Some("Donor Org".into()) })
        .with_receiver(OrgRef { ref_id: Some("BBB-222".into()), name: None })
        .with_currency_hint(CurrencyCode::from("EUR"));

    let mut activity = Activity::new("IATI-XYZ-12345");
    activity.transactions.push(tx);

    println!("{:?}", activity);
}
