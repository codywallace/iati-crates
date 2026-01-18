//! iati-types: strongly-typed, IO-free core models for IATI Activity v2.03.
//!
//! Other downstream crates (e.g. 'iati-xml', 'iati-transform') can provide parsing, serialization,
//! validation, and codelist lookups.

pub mod money;
pub mod tx;

pub use money::{CurrencyCode, Money};
pub use tx::{Transaction, TxType};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Lightweight organisation reference used here through the Activity tree.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))] 
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OrgRef {
    /// This is the IATI identifier of the organisation or registry-specific id.
    pub ref_id: Option<String>,
    /// Display name (narrative text, typically first/default language).
    pub name: Option<String>,
}

/// IATI Activity (trimmed to basic fields for foundational fields of the Activity struct here).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Activity {
    /// 'iati-identifier' element: the unique identifier for the Activity.
    pub iati_identifier: String,
    /// Default ISO 4217 currency for monety values in this Activity.
    /// Use when 'value/@currency' is not present.
    pub default_currency: Option<CurrencyCode>,
    /// Transactions recorded for this activity.
    pub transactions: Vec<Transaction>,
    /// Reporting organisation publishing this Activity.
    pub reporting_org: Option<OrgRef>,
    /// Activity start/end dates from 'activity-date' element.
    pub activity_start: Option<NaiveDate>,
    pub activity_end: Option<NaiveDate>,
}

impl Activity {
    pub fn new<S: Into<String>>(iati_identifier: S) -> Self {
        Self {
            iati_identifier: iati_identifier.into(),
            default_currency: None,
            transactions: Vec::new(),
            reporting_org: None,
            activity_start: None,
            activity_end: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::{Transaction, TxType};
    use rust_decimal::Decimal;

    #[test]
    fn tx_type_roundtrip() {
        assert_eq!(TxType::from(1).code(), 1);
        assert_eq!(TxType::from(13).code(), 13);
        assert!(matches!(TxType::from(99), TxType::Unknown(99)));
        assert_eq!("3".parse::<TxType>().unwrap().code(), 3);
    }

    #[test]
    fn money_uppercases_currency() {
        use crate::money::{CurrencyCode, Money};
        let m = Money {
            amount: Decimal::new(1000, 2), 
            currency: Some(CurrencyCode::from("usd")),
            value_date: None,
        };
        assert_eq!(m.currency.unwrap().0, "USD");
    }

    #[test]
    fn transaction_new_and_builders() {
        use crate::money::{CurrencyCode, Money};
        use chrono::NaiveDate;

        let date = NaiveDate::from_ymd_opt(2023, 5, 1).unwrap();
        let money = Money::new(Decimal::new(5000, 2));

        let tx = Transaction::new(TxType::Disbursement, date, money.clone())
            .with_provider(OrgRef {
                ref_id: Some("AAA-111".into()),
                name: Some("Donor Org".into()),
            })
            .with_receiver(OrgRef {
                ref_id: Some("BBB-222".into()),
                name: None,
            })
            .with_currency_hint(CurrencyCode::from("EUR"));

        assert_eq!(tx.tx_type, TxType::Disbursement);
        assert_eq!(tx.date, date);
        assert_eq!(tx.value.amount, money.amount);
        assert_eq!(
            tx.provider_org.as_ref().unwrap().ref_id.as_deref(),
            Some("AAA-111")
        );
        assert_eq!(
            tx.receiver_org.as_ref().unwrap().ref_id.as_deref(),
            Some("BBB-222")
        );
        assert_eq!(tx.currency_hint.as_ref().unwrap().0, "EUR");
    }
}
