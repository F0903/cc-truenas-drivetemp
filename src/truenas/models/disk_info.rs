use super::DiskQueryItem;

#[derive(Debug, Clone, PartialEq)]
pub struct DiskInfo {
    pub name: String,
    pub devname: Option<String>,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub pool: Option<String>,
    pub identifier: Option<String>,
}

impl DiskInfo {
    pub fn from_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            devname: None,
            model: None,
            serial: None,
            pool: None,
            identifier: None,
        }
    }
}

impl From<DiskQueryItem> for DiskInfo {
    fn from(value: DiskQueryItem) -> Self {
        Self {
            name: value.name,
            devname: value.devname,
            model: value.model,
            serial: value.serial,
            pool: value.pool,
            identifier: value.identifier,
        }
    }
}
