use crate::polling::TempChannel;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub(crate) struct CachedState {
    pub temps: Vec<TempChannel>,
    pub last_updated: Option<SystemTime>,
    pub last_error: Option<String>,
    pub missing: Vec<String>,
}
