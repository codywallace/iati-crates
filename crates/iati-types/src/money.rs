use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// ISO 4217 currency code stored as uppercase string.
/// Kept as a newtype to allow lightweight validation/normalisation later.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))] 
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CurrencyCode(pub String);

impl From<&str> for CurrencyCode {  
    fn from(s: &str) -> Self {
        Self(s.to_ascii_uppercase()) 
    }
}
impl From<String> for CurrencyCode {
    fn from(s: String) -> Self {  
        Self(s.to_ascii_uppercase())
    }
}

/// Monetary amount with currency and value-date.
/// In IATI, '<value>' carries '@currency' and '@value-date' attributes.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Money {
    pub amount: Decimal,
    /// If 'None', callers may fall back to 'Activity.default_currency'.
    pub currency: Option<CurrencyCode>,
    /// Preferred date to use for FX. If 'None', callers may fall back to
    /// 'Transaction.transaction_date' or 'Activity.activity_start'.
    pub value_date: Option<NaiveDate>, 
}

impl Money { 
    pub fn new(amount: Decimal) -> Self {
        Self {
            amount,
            currency: None,
            value_date: None,
        }
    }
}
