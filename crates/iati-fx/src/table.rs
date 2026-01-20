use std::collections::BTreeMap;

use chrono::NaiveDate;
use chrono::Datelike;
use iati_types::CurrencyCode;
use rust_decimal::Decimal;

use crate::provider::{FxError, FxProvider};

/// Year + month (IMF data is monthly).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct YearMonth {
    pub year: i32,
    pub month: u32,
}

impl YearMonth {
    pub fn from_date(date: NaiveDate) -> Self {
        YearMonth {
            year: date.year(),
            month: date.month(),
        }
    }
}

/// Structure holding IMF monthly "domestic currency per USD" exchange rates.
#[derive(Debug, Clone)]
pub struct FxTable {
    /// Map ((currency, YearMonth) -> rate)
    pub ncu_per_usd: BTreeMap<(CurrencyCode, YearMonth), Decimal>, 
}

impl FxTable {
    pub fn new() -> Self {
        FxTable {
            ncu_per_usd: BTreeMap::new(),
        }
    }

    /// Get the exchange rate for currency to USD for the given year and month.
    fn get_monthly_usd_rate(
        &self,
        code: &CurrencyCode,
        date: NaiveDate,
    ) -> Result<Decimal, FxError> {
        let ym: YearMonth = YearMonth::from_date(date);
        self.ncu_per_usd
            .get(&(code.clone(), ym))
            .cloned()
            .ok_or_else(|| FxError::MissingRate(code.clone(), date))
    }
}

impl FxProvider for FxTable {
    fn get_rate(
        &self,
        source_currency: &CurrencyCode,
        target_currency: &CurrencyCode,
        date: NaiveDate,
    ) -> Result<Decimal, FxError> {
        if source_currency == target_currency {
            return Ok(Decimal::ONE);
        }

        // IMF: rate = NCU per USD
        let r_from: Decimal = self.get_monthly_usd_rate(source_currency, date)?;
        let r_to: Decimal = self.get_monthly_usd_rate(target_currency, date)?;

        // Cross rate:
        //   1 from = (r_to / r_from) to 
        Ok(r_to / r_from)
    }
}