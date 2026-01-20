use chrono::NaiveDate;
use iati_types::{Activity, CurrencyCode, Money};
use rust_decimal::Decimal;

use crate::provider::{FxError, FxProvider};

/// Follows IATI logic: use explicit transaction currency, else fall back to activity.default_currency.
pub fn resolve_source_currency(
    money: &Money,
    activity_default: Option<&CurrencyCode>,
) -> Option<CurrencyCode> {
    money
        .currency
        .clone()
        .or_else(|| activity_default.cloned())
}

/// Convert a Money amount to a target currency.
pub fn convert_money(
    money: &Money,
    activity_default: Option<&CurrencyCode>,
    target: &CurrencyCode,
    value_date: Option<NaiveDate>,
    fx: &impl FxProvider,
) -> Result<Money, FxError> {
    let src: CurrencyCode = resolve_source_currency(money, activity_default)
        .ok_or(FxError::UnsupportedCurrency(CurrencyCode::from("UNKNOWN")))?;

    let date: NaiveDate = value_date.ok_or(FxError::MissingDate)?;

    let rate: Decimal = fx.get_rate(&src, target, date)?;

    let new_amount: Decimal = money.amount * rate;

    Ok(Money {
        amount: new_amount,
        currency: Some(target.clone()),
        value_date: money.value_date,
    })
}

/// Convert an entire Activity, producing a new Activity with converted amounts. 
pub fn convert_activity(
    activity: &Activity,
    target: &CurrencyCode,
    fx: &impl FxProvider,
) -> Result<Activity, FxError> {
    let mut out: Activity = activity.clone();
    out.default_currency = Some(target.clone());

    for tx in &mut out.transactions {
        let date: Option<NaiveDate> = tx.value.value_date.or(Some(tx.date));

        let conv: Money = convert_money(
            &tx.value,
            activity.default_currency.as_ref(),
            target,
            date,
            fx,
        )?;

        tx.value = conv;
    }
    Ok(out)
}
