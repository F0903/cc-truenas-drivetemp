use super::*;

#[test]
fn config_defaults_are_applied() {
    let config: PluginConfig = toml::from_str(
        r#"
            [truenas]
            url = "wss://nas/api/current"
            api_key = "secret"
        "#,
    )
    .unwrap();

    assert_eq!(config.poll_interval_seconds, 300);
    assert_eq!(
        config.truenas.api_key_env.as_deref(),
        Some("TRUENAS_API_KEY")
    );
    assert_eq!(config.truenas.timeout_seconds, 20);
}

#[test]
fn load_normalizes_lists_and_strings() {
    let mut config: PluginConfig = toml::from_str(
        r#"
            disks = [" sdb ", "sda", "sda", ""]

            [truenas]
            url = "  wss://nas/api/current  "
            username = " admin "
            api_key = " secret "
            api_key_env = ""
        "#,
    )
    .unwrap();

    config.normalize();
    config.validate().unwrap();

    assert_eq!(config.truenas.url, "wss://nas/api/current");
    assert_eq!(config.truenas.username.as_deref(), Some("admin"));
    assert_eq!(config.truenas.api_key.as_deref(), Some("secret"));
    assert_eq!(config.truenas.api_key_env, None);
    assert_eq!(config.disks, vec!["sda", "sdb"]);
}
