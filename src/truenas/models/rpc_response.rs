use super::RpcError;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub(in crate::truenas) struct RpcResponse {
    pub(in crate::truenas) id: Option<u64>,
    pub(in crate::truenas) result: Option<Value>,
    pub(in crate::truenas) error: Option<RpcError>,
}
