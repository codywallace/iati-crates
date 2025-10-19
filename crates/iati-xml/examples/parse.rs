use std::io::{self, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read full XML from stdin
    let mut xml = String::new();
    io::stdin().read_to_string(&mut xml)?;

    let s = xml.as_str();


    let output = if s.contains("<iati-activities") {
        // Full document with 1+ activities
        let activities = iati_xml::parse_activities(s)?;
        serde_json::to_string_pretty(&activities)?
    } else if s.contains("<iati-activity") {
        // Single activity fragment
        let activity = iati_xml::parse_activity(s)?;
        serde_json::to_string_pretty(&activity)?
    } else {
        // Fallback: try parsing as full doc, then as single
        match iati_xml::parse_activities(s) {
            Ok(acts) if !acts.is_empty() => serde_json::to_string_pretty(&acts)?,
            _ => {
                let activity = iati_xml::parse_activity(s)?;
                serde_json::to_string_pretty(&activity)?
            }
        }
    };

    println!("{output}");
    Ok(())
}
