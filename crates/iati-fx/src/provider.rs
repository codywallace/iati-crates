use chrono::NaiveDate;
use iati_types::CurrencyCode;
use rust_decimal::Decimal;
use thiserror::Error;


#[derive(Debug, Error)]
pub enum FxError {
    #[error("No FX rate available for {0:?} on {1}")]
    MissingRate(CurrencyCode, NaiveDate),

    #[error("Unsupported currency: {0:?}")]
    UnsupportedCurrency(CurrencyCode),

    #[error("Target currency {0:?} not found in FX table")]
    UnsupportedTarget(CurrencyCode),

    #[error("Date missing for conversion (need activity or transaction value_date)")]
    MissingDate,
}

pub trait FxProvider {
    fn get_rate(
        &self,
        source_currency: &CurrencyCode,
        target_currency: &CurrencyCode,
        date: NaiveDate,
    ) -> Result<Decimal, FxError>;
}