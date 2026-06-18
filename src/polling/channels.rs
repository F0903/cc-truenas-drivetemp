use super::TempChannel;
use crate::truenas::DriveTemperature;

pub fn to_channels(temperatures: &[DriveTemperature]) -> Vec<TempChannel> {
    let mut channels = Vec::new();
    for temp in temperatures {
        channels.push(TempChannel {
            id: drive_id(&temp.name),
            label: drive_label(temp),
            number: next_channel_number(&channels),
            celsius: temp.celsius,
        });
    }

    if !temperatures.is_empty() {
        append_aggregate_channels(&mut channels, temperatures);
    }

    channels
}

pub fn failsafe_aggregate_max_channel(celsius: f64, number: u32) -> TempChannel {
    TempChannel {
        id: aggregate_id("max"),
        label: "TrueNAS drives max fail-safe".to_string(),
        number,
        celsius,
    }
}

pub fn aggregate_id(kind: &str) -> String {
    format!("aggregate_{}", safe_id(kind))
}

pub fn safe_id(value: &str) -> String {
    let mut out = String::new();
    for ch in value.trim().chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    let trimmed = out.trim_matches(['.', '_']);
    if trimmed.is_empty() {
        "unnamed".to_string()
    } else {
        trimmed.to_string()
    }
}

fn append_aggregate_channels(channels: &mut Vec<TempChannel>, temperatures: &[DriveTemperature]) {
    let max = temperatures
        .iter()
        .map(|temp| temp.celsius)
        .fold(f64::NEG_INFINITY, f64::max);
    let min = temperatures
        .iter()
        .map(|temp| temp.celsius)
        .fold(f64::INFINITY, f64::min);
    let avg = temperatures.iter().map(|temp| temp.celsius).sum::<f64>() / temperatures.len() as f64;

    for (kind, label, celsius) in [
        ("max", "TrueNAS drives max", max),
        ("min", "TrueNAS drives min", min),
        ("avg", "TrueNAS drives average", avg),
    ] {
        channels.push(TempChannel {
            id: aggregate_id(kind),
            label: label.to_string(),
            number: next_channel_number(channels),
            celsius,
        });
    }
}

fn drive_id(name: &str) -> String {
    format!("drive_{}", safe_id(name))
}

fn drive_label(temp: &DriveTemperature) -> String {
    let Some(metadata) = &temp.metadata else {
        return format!("Drive {}", temp.name);
    };
    match (&metadata.model, &metadata.pool) {
        (Some(model), Some(pool)) => format!("Drive {} ({model}, {pool})", temp.name),
        (Some(model), None) => format!("Drive {} ({model})", temp.name),
        (None, Some(pool)) => format!("Drive {} ({pool})", temp.name),
        (None, None) => format!("Drive {}", temp.name),
    }
}

fn next_channel_number(channels: &[TempChannel]) -> u32 {
    channels.len() as u32 + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::truenas::{DiskInfo, DriveTemperature};

    #[test]
    fn safe_id_replaces_unsafe_chars() {
        assert_eq!(safe_id("nvme0n1"), "nvme0n1");
        assert_eq!(safe_id("disk/name with spaces"), "disk_name_with_spaces");
        assert_eq!(safe_id("..."), "unnamed");
    }

    #[test]
    fn channel_conversion_adds_drive_and_aggregate_temps() {
        let temps = vec![
            DriveTemperature {
                name: "sda".to_string(),
                celsius: 34.2,
                metadata: Some(DiskInfo {
                    name: "sda".to_string(),
                    devname: None,
                    model: Some("Model A".to_string()),
                    serial: None,
                    pool: Some("tank".to_string()),
                    identifier: None,
                }),
            },
            DriveTemperature {
                name: "nvme0n1".to_string(),
                celsius: 44.0,
                metadata: None,
            },
        ];

        let channels = to_channels(&temps);
        assert_eq!(channels.len(), 5);
        assert_eq!(channels[0].id, "drive_sda");
        assert_eq!(channels[0].label, "Drive sda (Model A, tank)");
        assert_eq!(channels[2].id, "aggregate_max");
        assert_eq!(channels[2].celsius, 44.0);
        assert_eq!(channels[4].id, "aggregate_avg");
        assert!((channels[4].celsius - 39.1).abs() < f64::EPSILON);
    }
}
