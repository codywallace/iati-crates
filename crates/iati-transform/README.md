# iati-transform

Transformations and rollups for [`iati-types`](https://crates.io/crates/iati-types) `Activity` data:

- Sum transactions by type / year / currency
- Currency fallback logic (use `value.currency` else `activity.default_currency`)
- IO-free; pure functions you can plug into CLI, services, or data pipelines

## Example

```rust
use iati_transform::{aggregate_by_type, aggregate_by_year_and_type, FxCurrency};
use iati_types::{Activity, TxType, Transaction, Money, CurrencyCode};
use chrono::NaiveDate;
use rust_decimal::Decimal;

let mut a = Activity::new("ACT-1");
a.default_currency = Some(CurrencyCode::from("USD"));
a.transactions.push(Transaction::new(
    TxType::Disbursement,
    NaiveDate::from_ymd_opt(2023,1,1).unwrap(),
    Money { amount: Decimal::new(1000,2), currency: None, value_date: None }
));
a.transactions.push(Transaction::new(
    TxType::Disbursement,
    NaiveDate::from_ymd_opt(2023,2,1).unwrap(),
    Money { amount: Decimal::new(500,2), currency: Some(CurrencyCode::from("EUR")), value_date: None }
));

// Sum by type in native currency space (no FX)
let sums = aggregate_by_type(&[a], FxCurrency::Native);
assert!(sums.total_for(TxType::Disbursement, &CurrencyCode::from("USD")).is_some());
