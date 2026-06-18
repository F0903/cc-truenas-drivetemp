use anyhow::Result;
use log::LevelFilter;
use std::str::FromStr;

const ENV_CC_LOG: &str = "CC_LOG";

pub(super) fn setup_logging(debug: bool) -> Result<()> {
    let log_level = if debug {
        LevelFilter::Debug
    } else if let Ok(log_lvl) = std::env::var(ENV_CC_LOG) {
        LevelFilter::from_str(&log_lvl).unwrap_or(LevelFilter::Info)
    } else {
        LevelFilter::Info
    };

    if systemd_journal_logger::connected_to_journal() {
        use crate::VERSION;
        use systemd_journal_logger::JournalLog;

        JournalLog::new()?
            .with_extra_fields(vec![("VERSION", VERSION)])
            .install()?;
        log::set_max_level(log_level);
        return Ok(());
    }

    env_logger::Builder::new().filter_level(log_level).init();
    Ok(())
}
