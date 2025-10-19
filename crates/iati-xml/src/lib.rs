use chrono::NaiveDate;
use iati_types::{
    money::{CurrencyCode, Money},
    tx::{Transaction, TxType},
    Activity,
};
use quick_xml::{
    events::{attributes::Attributes, Event},
    name::QName,
    Reader, Writer,
};

use rust_decimal::Decimal;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("XML attribute error: {0}")]
    XmlAttr(#[from] quick_xml::events::attributes::AttrError),
    #[error("encoding error: {0}")]
    Encoding(#[from] quick_xml::encoding::EncodingError),
    #[error("io error: {0}")]                // <-- add this line
    Io(#[from] std::io::Error),              // <-- and this line
    #[error("missing required field: {0}")]
    Missing(&'static str),
    #[error("invalid decimal: {0}")]
    Decimal(#[from] rust_decimal::Error),
    #[error("invalid integer: {0}")]
    Int(#[from] std::num::ParseIntError),
    #[error("invalid date: {0}")]
    Date(#[from] chrono::ParseError),
}


/// Builder used while reading a <transaction>
#[derive(Default)]
struct TxBuild {
    tx_type: Option<TxType>,
    date: Option<NaiveDate>,
    amount: Option<Decimal>,
    value_currency: Option<CurrencyCode>,
    value_date: Option<NaiveDate>,
}

fn parse_tx_type(mut attrs: Attributes<'_>, tx_build: &mut Option<TxBuild>) -> Result<(), ParseError> {
    let mut code: Option<u16> = None;
    for a in attrs.with_checks(false) {
        let a = a?;
        if a.key == QName(b"code") {
            code = Some(a.unescape_value()?.into_owned().parse()?);
        }
    }
    if let Some(b) = tx_build.as_mut() {
        b.tx_type = Some(TxType::from(
            code.ok_or(ParseError::Missing("transaction-type/@code"))?,
        ));
    }
    Ok(())
}

fn parse_tx_date(mut attrs: Attributes<'_>, tx_build: &mut Option<TxBuild>) -> Result<(), ParseError> {
    let mut iso: Option<String> = None;
    for a in attrs.with_checks(false) {
        let a = a?;
        if a.key == QName(b"iso-date") {
            iso = Some(a.unescape_value()?.into_owned());
        }
    }
    if let Some(b) = tx_build.as_mut() {
        let iso = iso.ok_or(ParseError::Missing("transaction-date/@iso-date"))?;
        b.date = Some(NaiveDate::parse_from_str(&iso, "%Y-%m-%d")?);
    }
    Ok(())
}

/// Parse a single `<iati-activity>` fragment and its `<transaction>` children into an `Activity`.
pub fn parse_activity(xml: &str) -> Result<Activity, ParseError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true); // quick-xml 0.38+

    let mut buf = Vec::new();
    let mut current_text: Option<String> = None;

    // Activity fields
    let mut default_currency: Option<CurrencyCode> = None;
    let mut iati_identifier: Option<String> = None;
    let mut transactions: Vec<Transaction> = Vec::new();

    let mut tx_build: Option<TxBuild> = None;

    loop {
        match reader.read_event_into(&mut buf)? {
            // ---------- <start> ----------
            Event::Start(e) => match e.name().as_ref() {
                b"iati-activity" => {
                    for a in e.attributes().with_checks(false) {
                        let a = a?;
                        if a.key == QName(b"default-currency") {
                            default_currency = Some(CurrencyCode(a.unescape_value()?.into_owned()));
                        }
                    }
                }
                b"iati-identifier" => {
                    current_text = Some(String::new());
                }
                b"transaction" => {
                    tx_build = Some(TxBuild::default());
                }
                b"transaction-type" => {
                    parse_tx_type(e.attributes(), &mut tx_build)?;
                }
                b"transaction-date" => {
                    parse_tx_date(e.attributes(), &mut tx_build)?;
                }
                b"value" => {
                    current_text = Some(String::new()); // capture text later
                    if let Some(b) = tx_build.as_mut() {
                        for a in e.attributes().with_checks(false) {
                            let a = a?;
                            if a.key == QName(b"currency") {
                                b.value_currency =
                                    Some(CurrencyCode::from(a.unescape_value()?.into_owned()));
                            }
                            if a.key == QName(b"value-date") {
                                let s = a.unescape_value()?.into_owned();
                                b.value_date = Some(NaiveDate::parse_from_str(&s, "%Y-%m-%d")?);
                            }
                        }
                    }
                }
                _ => {}
            },

            // ---------- <empty/> ----------
            Event::Empty(e) => match e.name().as_ref() {
                b"transaction-type" => {
                    parse_tx_type(e.attributes(), &mut tx_build)?;
                }
                b"transaction-date" => {
                    parse_tx_date(e.attributes(), &mut tx_build)?;
                }
                b"value" => {
                    // Handle `<value .../>` as empty (no amount text)
                    if let Some(b) = tx_build.as_mut() {
                        for a in e.attributes().with_checks(false) {
                            let a = a?;
                            if a.key == QName(b"currency") {
                                b.value_currency =
                                    Some(CurrencyCode::from(a.unescape_value()?.into_owned()));
                            }
                            if a.key == QName(b"value-date") {
                                let s = a.unescape_value()?.into_owned();
                                b.value_date = Some(NaiveDate::parse_from_str(&s, "%Y-%m-%d")?);
                            }
                        }
                    }
                }
                _ => {}
            },

            // ---------- text nodes ----------
            Event::Text(t) => {
                if let Some(s) = current_text.as_mut() {
                    s.push_str(&t.decode()?); // Cow<str> -> &str
                }
            }

            // ---------- </end> ----------
            Event::End(e) => match e.name().as_ref() {
                b"iati-identifier" => {
                    let val = current_text.take().unwrap_or_default();
                    iati_identifier = Some(val.trim().to_string());
                }
                b"value" => {
                    if let Some(b) = tx_build.as_mut() {
                        let val = current_text.take().unwrap_or_default();
                        let dec = Decimal::from_str(val.trim())?;
                        b.amount = Some(dec);
                    }
                }
                b"transaction" => {
                    if let Some(b) = tx_build.take() {
                        let tx_type = b.tx_type.ok_or(ParseError::Missing("transaction-type"))?;
                        let date = b.date.ok_or(ParseError::Missing("transaction-date/@iso-date"))?;
                        let mut money = Money::new(b.amount.ok_or(ParseError::Missing("value"))?);
                        money.currency = b.value_currency;
                        money.value_date = b.value_date;
                        let tx = Transaction::new(tx_type, date, money);
                        transactions.push(tx);
                    }
                }
                _ => {}
            },

            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    let id = iati_identifier.ok_or(ParseError::Missing("iati-identifier"))?;
    let mut activity = Activity::new(id);
    activity.default_currency = default_currency;
    activity.transactions = transactions;
    Ok(activity)
}

/// Parse a full `<iati-activities>` document, returning one `Activity` per `<iati-activity>`.
pub fn parse_activities(xml: &str) -> Result<Vec<Activity>, ParseError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut activities: Vec<Activity> = Vec::new();

    // When we enter an <iati-activity>, capture all events until its matching </iati-activity>
    let mut depth: usize = 0;
    let mut writer: Option<Writer<Vec<u8>>> = None;

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) if e.name().as_ref() == b"iati-activity" => {
                depth = 1;
                let mut w = Writer::new(Vec::<u8>::new());
                w.write_event(Event::Start(e.to_owned()))?;
                writer = Some(w);
            }
            Event::Empty(e) if e.name().as_ref() == b"iati-activity" => {
                // Rare degenerate case: <iati-activity .../> (no children)
                let mut w = Writer::new(Vec::<u8>::new());
                w.write_event(Event::Empty(e.to_owned()))?;
                let bytes = w.into_inner();
                let frag = String::from_utf8(bytes).unwrap_or_default();
                activities.push(parse_activity(&frag)?);
            }
            ev @ Event::Start(_) | ev @ Event::Empty(_) | ev @ Event::Text(_) | ev @ Event::CData(_) | ev @ Event::Comment(_) | ev @ Event::PI(_) => {
                if let Some(w) = writer.as_mut() {
                    if let Event::Start(ref e) = ev {
                        if e.name().as_ref() == b"iati-activity" {
                            depth += 1;
                        }
                    }
                    w.write_event(ev.to_owned())?;
                }
            }
            Event::End(e) => {
                if let Some(w) = writer.as_mut() {
                    w.write_event(Event::End(e.to_owned()))?;
                    if e.name().as_ref() == b"iati-activity" {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            let bytes = writer.take().unwrap().into_inner();
                            let frag = String::from_utf8(bytes).unwrap_or_default();
                            activities.push(parse_activity(&frag)?);
                        }
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(activities)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn parse_minimal_activity_with_one_tx() {
        let xml = r#"
        <iati-activity default-currency="USD">
            <iati-identifier>IATI-XYZ-12345</iati-identifier>
            <transaction>
                <transaction-type code="3"/>
                <transaction-date iso-date="2023-05-01"/>
                <value currency="EUR" value-date="2023-05-02">50.00</value>
            </transaction>
        </iati-activity>
        "#;

        let act = parse_activity(xml).expect("parsed");
        assert_eq!(act.iati_identifier, "IATI-XYZ-12345");
        assert_eq!(act.default_currency.as_ref().unwrap().0, "USD");
        assert_eq!(act.transactions.len(), 1);

        let tx = &act.transactions[0];
        assert!(matches!(tx.tx_type, TxType::Disbursement));
        assert_eq!(tx.date, NaiveDate::from_ymd_opt(2023, 5, 1).unwrap());
        assert_eq!(tx.value.amount, Decimal::new(5000, 2));
        assert_eq!(tx.value.currency.as_ref().unwrap().0, "EUR");
        assert_eq!(
            tx.value.value_date.unwrap(),
            NaiveDate::from_ymd_opt(2023, 5, 2).unwrap()
        );
    }

    #[test]
    fn parse_two_activities_from_wrapper() {
        let xml = r#"
        <iati-activities version="2.03">
            <iati-activity default-currency="USD">
                <iati-identifier>ACT-1</iati-identifier>
                <transaction>
                    <transaction-type code="3"/>
                    <transaction-date iso-date="2023-01-01"/>
                    <value currency="USD" value-date="2023-01-01">10.00</value>
                </transaction>
            </iati-activity>
            <iati-activity default-currency="EUR">
                <iati-identifier>ACT-2</iati-identifier>
                <transaction>
                    <transaction-type code="4"/>
                    <transaction-date iso-date="2023-02-02"/>
                    <value currency="EUR" value-date="2023-02-03">20.00</value>
                </transaction>
            </iati-activity>
        </iati-activities>
        "#;

        let acts = parse_activities(xml).expect("parsed doc");
        assert_eq!(acts.len(), 2);
        assert_eq!(acts[0].iati_identifier, "ACT-1");
        assert_eq!(acts[1].iati_identifier, "ACT-2");
    }

    #[test]
    fn empty_value_element_errors() {
        // <value .../> without amount text should yield Missing("value")
        let xml = r#"
        <iati-activity>
            <iati-identifier>ACT-EMPTY</iati-identifier>
            <transaction>
                <transaction-type code="3"/>
                <transaction-date iso-date="2023-05-01"/>
                <value currency="USD" value-date="2023-05-01"/>
            </transaction>
        </iati-activity>
        "#;

        let err = parse_activity(xml).unwrap_err();
        assert!(matches!(err, ParseError::Missing("value")));
    }
}
