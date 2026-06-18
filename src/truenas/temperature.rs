use super::models::{DiskInfo, DriveTemperature};
use anyhow::{Result, bail};
use serde_json::Value;
use std::collections::HashMap;

pub(super) fn parse_temperature_result(
    raw: &Value,
    metadata: &HashMap<String, DiskInfo>,
) -> Result<(Vec<DriveTemperature>, Vec<String>)> {
    let Some(map) = raw.as_object() else {
        bail!(
            "disk.temperatures returned {}, expected object",
            raw_type(raw)
        );
    };

    let mut temperatures = Vec::new();
    let mut missing = Vec::new();

    for (name, value) in map {
        if let Some(celsius) = extract_celsius(value) {
            temperatures.push(DriveTemperature {
                name: name.clone(),
                celsius,
                metadata: metadata.get(name).cloned(),
            });
        } else {
            missing.push(name.clone());
        }
    }

    temperatures.sort_by(|left, right| left.name.cmp(&right.name));
    missing.sort();
    Ok((temperatures, missing))
}

fn extract_celsius(value: &Value) -> Option<f64> {
    match value {
        Value::Null | Value::Bool(_) => None,
        Value::Number(number) => number.as_f64().filter(|value| value.is_finite()),
        Value::String(text) => text.parse::<f64>().ok().filter(|value| value.is_finite()),
        Value::Array(_) => None,
        Value::Object(map) => ["temperature", "temp", "current", "value"]
            .iter()
            .find_map(|key| map.get(*key).and_then(extract_celsius)),
    }
}

fn raw_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_temperature_shapes() {
        assert_eq!(extract_celsius(&json!(40)), Some(40.0));
        assert_eq!(extract_celsius(&json!("41.5")), Some(41.5));
        assert_eq!(extract_celsius(&json!({"temperature": 42})), Some(42.0));
        assert_eq!(extract_celsius(&json!({"temp": {"value": 43}})), Some(43.0));
        assert_eq!(extract_celsius(&json!(null)), None);
        assert_eq!(extract_celsius(&json!("n/a")), None);
        assert_eq!(extract_celsius(&json!("NaN")), None);
    }

    #[test]
    fn parses_temperature_result_and_missing_disks() {
        let mut metadata = HashMap::new();
        metadata.insert(
            "sda".to_string(),
            DiskInfo {
                name: "sda".to_string(),
                devname: Some("/dev/sda".to_string()),
                model: Some("Model A".to_string()),
                serial: None,
                pool: None,
                identifier: None,
            },
        );

        let (temperatures, missing) =
            parse_temperature_result(&json!({"sda": {"temperature": 36}, "sdb": null}), &metadata)
                .unwrap();

        assert_eq!(temperatures.len(), 1);
        assert_eq!(temperatures[0].name, "sda");
        assert_eq!(temperatures[0].celsius, 36.0);
        assert_eq!(
            temperatures[0].metadata.as_ref().unwrap().model.as_deref(),
            Some("Model A")
        );
        assert_eq!(missing, vec!["sdb"]);
    }
}
