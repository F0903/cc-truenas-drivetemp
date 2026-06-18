use super::TempChannel;

pub(super) struct PollSnapshot {
    pub(super) temps: Vec<TempChannel>,
    pub(super) missing: Vec<String>,
}
