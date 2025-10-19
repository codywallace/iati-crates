# iati-xml

XML parser for the [International Aid Transparency Initiative (IATI)](https://iatistandard.org) Activity Standard v2.03.

This crate provides a lightweight, streaming XML parser that converts `<iati-activity>` or `<iati-activities>` XML fragments into strongly-typed Rust structures defined in [`iati-types`](https://crates.io/crates/iati-types).

---

## Overview

The IATI Standard publishes aid and development finance data in XML.  
`iati-xml` bridges the gap between this XML and the Rust type system by exposing functions to parse full or partial activity data.

- **`parse_activity()`** parses a single `<iati-activity>`.
- **`parse_activities()`** parses an entire `<iati-activities>` document containing multiple activities.

The crate is designed to be:
- **Streaming-safe** (uses [`quick-xml`](https://crates.io/crates/quick-xml))
- **Zero-alloc where possible**
- **Compatible with `serde` models** from `iati-types`
- **Schema-friendly**, supporting key elements such as:
  - `iati-identifier`
  - `default-currency`
  - `transaction-type`, `transaction-date`, and `value`

---

## Example

```rust
use iati_xml::parse_activity;
use chrono::NaiveDate;
use rust_decimal::Decimal;

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

let activity = parse_activity(xml).expect("parsed");
println!("Parsed identifier: {}", activity.iati_identifier);
