# iati-crates

**Rust-based toolkit for working with International Aid Transparency Initiative (IATI) data**

---

## Overview

The **International Aid Transparency Initiative (IATI)** provides a global standard for publishing information on development and humanitarian resources. Thousands of organizations publish their aid activities in XML, but the data can be complex: nested structures, diverse transaction types, multiple currencies, and inconsistent reporting practices.

**iati-crates** is a Rust mono-repo providing modular crates to make IATI data easier to parse, clean, and analyze.

### Why this matters

- **Transparency & accountability** — Stronger, more usable aid data supports better oversight, decision-making, and trust in development cooperation.  
- **Consistency** — Raw XML can be difficult to work with; strongly typed Rust models reduce errors and improve reproducibility.  
- **Performance & safety** — Rust’s speed and type safety make it ideal for processing millions of activity records.  
- **Flexibility** — Use individual crates as building blocks, or the umbrella crate `iati-crates` for an all-in-one solution.

---

## Crates in this workspace

- **`iati-types`**  
  Core data types (`Activity`, `Transaction`, `Money`, `TxType`, etc.), IO-free and strongly typed.

- **`iati-xml`** *(planned)*  
  XML parsing and serialization (powered by `quick-xml` + `serde`).

- **`iati-transform`** *(planned)*  
  Cleaning, normalization, transaction rollups, and data quality checks.

- **`iati-fx`** *(planned)*  
  Currency conversion to USD or other targets, with pluggable FX providers.

- **`iati-cli`** *(planned)*  
  Command-line tool for validation, transformation, rollups, and export.

- **`iati-crates` (umbrella)** *(planned)*  
  A convenience crate that re-exports the others for easy “one-stop” use.

---

## Example (using `iati-types`)

```rust
use iati_types::{Activity, Transaction, TxType, Money, CurrencyCode, OrgRef};
use chrono::NaiveDate;
use rust_decimal::Decimal;

fn main() {
    let date = NaiveDate::from_ymd_opt(2023, 5, 1).unwrap();
    let money = Money::new(Decimal::new(5000, 2));

    let tx = Transaction::new(TxType::Disbursement, date, money)
        .with_provider(OrgRef { ref_id: Some("AAA-111".into()), name: Some("Donor Org".into()) })
        .with_currency_hint(CurrencyCode::from("EUR"));

    let mut activity = Activity::new("IATI-XYZ-12345");
    activity.transactions.push(tx);

    println!("{:?}", activity);
}
