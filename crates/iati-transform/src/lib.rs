
use chrono::Datelike;
use iati_types::{money::CurrencyCode, tx::TxType, Activity};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error; 

#[derive(Debug, Error)]
pub enum TransformError {
    #[error("missing currency (no value currency and no activity default)")]
    MissingCurrency,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FxCurrency {
    /// Keep each transaction's native currency (value.currency or activity.default_currency).
    Native,
    /// Convert everything to target currency with a placeholder 1:1 rate for now.
    /// (Real FX will be provided by a future iati-fx crate.)
    Fixed { target: CurrencyCode },
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ByTypeAndCurrency {
    // TxType -> Currency -> Decimal
    pub sums: BTreeMap<TxType, BTreeMap<CurrencyCode, Decimal>>,
}

impl ByTypeAndCurrency {
    pub fn add(&mut self, tx_type: TxType, currency: CurrencyCode, amount: Decimal) {
        self.sums
            .entry(tx_type)
            .or_default()
            .entry(currency)
            .and_modify(|x| *x += amount)
            .or_insert(amount);
    }

    pub fn total_for(&self, tx_type: TxType, currency: &CurrencyCode) -> Option<Decimal> {
        self.sums.get(&tx_type)?.get(currency).cloned()
    }
}

/// Resolve the currency for a transaction (value.currency or activity.default_currency).
fn resolve_currency(act: &Activity, currency: Option<CurrencyCode>) -> Result<CurrencyCode, TransformError> {
    match currency.or_else(|| act.default_currency.clone()) {
        Some(c) => Ok(c),
        None => Err(TransformError::MissingCurrency),
    }
}

/// Apply FX strategy. For now only Native or Fixed{target} with a 1:1 rate.
/// (A future iati-fx crate will supply actual FX conversions.)
fn apply_fx(_value_date: Option<chrono::NaiveDate>, amount: Decimal, from: &CurrencyCode, fx: &FxCurrency)
    -> (Decimal, CurrencyCode)
{
    match fx {
        FxCurrency::Native => (amount, from.clone()),
        FxCurrency::Fixed { target } => {
            // placeholder: 1:1 rate; swap to target currency
            (amount, target.clone())
        }
    }
}

/// Aggregate sums by TxType and Currency across many activities.
/// - Currency resolution: value.currency -> act.default_currency -> error.
/// - FX: Native (no conversion) or Fixed{target} (placeholder 1:1).
pub fn aggregate_by_type(activities: &[Activity], fx: FxCurrency) -> ByTypeAndCurrency {
    let mut out = ByTypeAndCurrency::default();

    for act in activities {
        for tx in &act.transactions {
            // resolve currency
            let src_cur = match resolve_currency(act, tx.value.currency.clone()) {
                Ok(c) => c,
                Err(_) => continue, // skip transactions without any currency info
            };

            let (amt, cur) = apply_fx(tx.value.value_date, tx.value.amount, &src_cur, &fx);
            out.add(tx.tx_type, cur, amt);
        }
    }

    out
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ByYearTypeAndCurrency {
    /// year -> TxType -> Currency -> Decimal
    pub sums: BTreeMap<i32, BTreeMap<TxType, BTreeMap<CurrencyCode, Decimal>>>,
}

impl ByYearTypeAndCurrency {
    pub fn add(&mut self, year: i32, tx_type: TxType, currency: CurrencyCode, amount: Decimal) {
        self.sums
            .entry(year)
            .or_default()
            .entry(tx_type)
            .or_default()
            .entry(currency)
            .and_modify(|x| *x += amount)
            .or_insert(amount);
    }
}

/// Aggregate by (year, type, currency). Uses `transaction.date.year()`.
pub fn aggregate_by_year_and_type(activities: &[Activity], fx: FxCurrency) -> ByYearTypeAndCurrency {
    let mut out = ByYearTypeAndCurrency::default();

    for act in activities {
        for tx in &act.transactions {
            let year = tx.date.year();
            let src_cur = match resolve_currency(act, tx.value.currency.clone()) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let (amt, cur) = apply_fx(tx.value.value_date, tx.value.amount, &src_cur, &fx);
            out.add(year, tx.tx_type, cur, amt);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use iati_types::{money::Money, tx::Transaction, Activity, TxType};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    fn mk_money(amount_cents: i64, currency: Option<&str>) -> Money {
        Money {
            amount: Decimal::new(amount_cents, 2),
            currency: currency.map(|c| CurrencyCode::from(c)),
            value_date: None,
        }
    }

    #[test]
    fn sum_by_type_native_currency() {
        let mut a = Activity::new("A1");
        a.default_currency = Some(CurrencyCode::from("USD"));

        a.transactions.push(Transaction::new(
            TxType::Disbursement,
            NaiveDate::from_ymd_opt(2023, 1, 10).unwrap(),
            mk_money(1000, None), // 10.00 USD (falls back to activity default)
        ));
        a.transactions.push(Transaction::new(
            TxType::Disbursement,
            NaiveDate::from_ymd_opt(2023, 2, 10).unwrap(),
            mk_money(500, Some("EUR")), // 5.00 EUR
        ));
        a.transactions.push(Transaction::new(
            TxType::OutgoingCommitment,
            NaiveDate::from_ymd_opt(2023, 2, 10).unwrap(),
            mk_money(700, Some("USD")), // 7.00 USD
        ));

        let sums = aggregate_by_type(&[a], FxCurrency::Native);
        assert_eq!(
            sums.total_for(TxType::Disbursement, &CurrencyCode::from("USD")).unwrap(),
            Decimal::new(1000, 2)
        );
        assert_eq!(
            sums.total_for(TxType::Disbursement, &CurrencyCode::from("EUR")).unwrap(),
            Decimal::new(500, 2)
        );
        assert_eq!(
            sums.total_for(TxType::OutgoingCommitment, &CurrencyCode::from("USD")).unwrap(),
            Decimal::new(700, 2)
        );
    }

    #[test]
    fn sum_by_year_and_type_fixed_target() {
        let mut a = Activity::new("A1");
        a.default_currency = Some(CurrencyCode::from("USD"));

        a.transactions.push(Transaction::new(
            TxType::Disbursement,
            NaiveDate::from_ymd_opt(2023, 1, 10).unwrap(),
            mk_money(1000, None), // 10.00 USD
        ));
        a.transactions.push(Transaction::new(
            TxType::Disbursement,
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            mk_money(500, Some("EUR")), // 5.00 EUR -> Fixed target GBP (1:1)
        ));

        let sums = aggregate_by_year_and_type(&[a], FxCurrency::Fixed { target: CurrencyCode::from("GBP") });
        use rust_decimal::prelude::ToPrimitive;
        // 2023: 10.00 -> GBP
        assert_eq!(
            sums.sums.get(&2023).unwrap()
                .get(&TxType::Disbursement).unwrap()
                .get(&CurrencyCode::from("GBP")).unwrap().to_f64().unwrap(),
            10.00_f64
        );
        // 2024: 5.00 -> GBP
        assert_eq!(
            sums.sums.get(&2024).unwrap()
                .get(&TxType::Disbursement).unwrap()
                .get(&CurrencyCode::from("GBP")).unwrap().to_f64().unwrap(),
            5.00_f64
        );
    }
}
