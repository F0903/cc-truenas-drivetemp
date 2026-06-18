#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TempChannel {
    pub id: String,
    pub label: String,
    pub number: u32,
    pub celsius: f64,
}
