use iati_fx::{FxTable, FxProvider};
use iati_types::{CurrencyCode};
use chrono::NaiveDate;
use rust_decimal::{Decimal, prelude::FromPrimitive};

#[test]
fn test_cross_rate() {
    let mut table = FxTable::new();

    // For March 2024:
    // 1 USD = 7.0 DKK
    // 1 USD = 0.9 EUR

    let ym = iati_fx::YearMonth { year: 2024, month: 3 };


    table.ncu_per_usd.insert((CurrencyCode::from("DKK"), ym), Decimal::new(70,1)); // 7.0
    table.ncu_per_usd.insert((CurrencyCode::from("EUR"), ym), Decimal::new(9,1));  // 0.9

    let date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();

    let rate = table.get_rate(&CurrencyCode::from("DKK"), &CurrencyCode::from("EUR"), date).unwrap();

    // expected rate: 0.9 / 7.0 = 0.12857142857
    let expected_rate = Decimal::from_f64(0.9 / 7.0).unwrap();

    assert!((rate - expected_rate).abs() < Decimal::new(1, 6)); // this means a difference of less than 0.000001
    assert_eq!(rate.round_dp(10), expected_rate.round_dp(10));  // this tests up to 10 decimal places in equality
}

