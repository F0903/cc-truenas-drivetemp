use super::DiskInfo;

#[derive(Debug, Clone, PartialEq)]
pub struct DriveTemperature {
    pub name: String,
    pub celsius: f64,
    pub metadata: Option<DiskInfo>,
}
