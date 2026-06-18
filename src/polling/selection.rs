use crate::config::PluginConfig;
use crate::truenas::DiskInfo;
use std::collections::{HashMap, HashSet};

pub(super) fn select_disk_names(
    metadata: &HashMap<String, DiskInfo>,
    config: &PluginConfig,
) -> Vec<String> {
    let mut names: Vec<String> = if config.disks.is_empty() {
        let exclude: HashSet<&str> = config.exclude_disks.iter().map(String::as_str).collect();
        metadata
            .keys()
            .filter(|name| !exclude.contains(name.as_str()))
            .cloned()
            .collect()
    } else {
        metadata.keys().cloned().collect()
    };
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    fn discovered_config() -> PluginConfig {
        toml::from_str(
            r#"
                exclude_disks = ["sdc"]

                [truenas]
                url = "wss://nas/api/current"
                api_key = "secret"
            "#,
        )
        .unwrap()
    }

    #[test]
    fn select_disk_names_excludes_from_discovered_disks() {
        let metadata = ["sda", "sdb", "sdc"]
            .into_iter()
            .map(|name| (name.to_string(), DiskInfo::from_name(name)))
            .collect();

        assert_eq!(
            select_disk_names(&metadata, &discovered_config()),
            vec!["sda", "sdb"]
        );
    }

    #[test]
    fn select_disk_names_uses_explicit_disks_without_exclusions() {
        let config: PluginConfig = toml::from_str(
            r#"
                disks = ["sda", "sdc"]
                exclude_disks = ["sdc"]

                [truenas]
                url = "wss://nas/api/current"
                api_key = "secret"
            "#,
        )
        .unwrap();
        let metadata = config
            .disks
            .iter()
            .map(|name| (name.clone(), DiskInfo::from_name(name)))
            .collect();

        assert_eq!(select_disk_names(&metadata, &config), vec!["sda", "sdc"]);
    }
}
