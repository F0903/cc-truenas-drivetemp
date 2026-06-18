use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(in crate::truenas) struct DiskQueryItem {
    pub(in crate::truenas) name: String,
    #[serde(default)]
    pub(in crate::truenas) devname: Option<String>,
    #[serde(default)]
    pub(in crate::truenas) model: Option<String>,
    #[serde(default)]
    pub(in crate::truenas) serial: Option<String>,
    #[serde(default)]
    pub(in crate::truenas) pool: Option<String>,
    #[serde(default)]
    pub(in crate::truenas) identifier: Option<String>,
}

impl DiskQueryItem {
    pub(in crate::truenas) fn name(&self) -> &str {
        &self.name
    }
}
