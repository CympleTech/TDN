use serde_derive::{Deserialize, Serialize};
use serde_json::json;

use crate::primitives::types::{BlockByte, GroupID, LevelPermissionByte, RPCParams};

/// Request in levels and local outside call.
/// Local RPC Format:
/// jsonrpc = {
///     "jsonrpc": "2.0",
///     "id": "0",
///     "method": "local",
///     "params": {
///         "xx": "...xxx..."
///     }
/// }
///
/// Upper RPC Format:
/// jsonrpc = {
///     "jsonrpc": "2.0",
///     "id": "0",
///     "method": "upper",
///     "params": {
///         "block": "xxxxxx"
///     }
/// }
///
/// Lower RPC Format:
/// jsonrpc = {
///     "jsonrpc": "2.0",
///     "id": "0",
///     "method": "lower",
///     "params": {
///         "block": "xxxxxx"
///     }
/// }
///
/// Permission RPC Format:
/// jsonrpc = {
///     "jsonrpc": "2.0",
///     "id": "0",
///     "method": "permission",
///     "params": {
///         "group": "xxxxxx",
///         "value": "xxxxxx",
///     }
/// }
///
/// use in rpc session and rpc
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(bound = "")]
pub enum Request {
    Local(GroupID, RPCParams),
    Upper(GroupID, BlockByte),
    Lower(GroupID, BlockByte),
    Permission(GroupID, LevelPermissionByte),
    Invalid,
}

impl Request {
    pub fn parse(method: &String, params: &RPCParams) -> Self {
        match method.as_str() {
            "local" => {
                let group = Default::default();
                Request::Local(group, params.clone())
            }
            "upper" => {
                let _block_bytes = params.get("block");
                let _group = params.get("group");
                let block_bytes = vec![];
                let group = Default::default();
                Request::Upper(group, block_bytes)
            }
            "lower" => {
                let _block_bytes = params.get("block");
                let _group = params.get("group");
                let block_bytes = vec![];
                let group = Default::default();
                Request::Lower(group, block_bytes)
            }
            "permission" => {
                let _permission = params.get("value");
                let _group = params.get("group");

                let permission = vec![];
                let group = Default::default();
                Request::Permission(group, permission)
            }
            _ => Request::Invalid,
        }
    }

    pub fn deparse(&self) -> (String, RPCParams) {
        match self {
            Request::Local(_group, params) => ("local".to_owned(), params.clone()),
            Request::Upper(group, params) => (
                "upper".to_owned(),
                json!({"block": params.clone(), "group": group.to_string()}),
            ),
            Request::Lower(group, params) => (
                "lower".to_owned(),
                json!({"block": params.clone(), "group": group.to_string()}),
            ),
            Request::Permission(group, params) => (
                "permission".to_owned(),
                json!({"value": params.clone(), "group": group.to_string()}),
            ),
            _ => ("invalid".into(), Default::default()),
        }
    }
}
