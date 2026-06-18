use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub(in crate::truenas) struct RpcError {
    pub(in crate::truenas) code: Option<Value>,
    pub(in crate::truenas) message: String,
}
