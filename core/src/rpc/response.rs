use serde_derive::{Deserialize, Serialize};
use serde_json::json;

use crate::primitives::types::{EventID, GroupID, RPCParams};

/// Response in levels and local outside call.
/// Local RPC Format:
/// jsonrpc = {
///     "jsonrpc": "2.0",
///     "id": "0",
///     "method": "local",
///     "result": "_RPCParams_"
/// }
///
/// Upper RPC Format:
/// jsonrpc = {
///     "jsonrpc": "2.0",
///     "id": "0",
///     "method": "upper",
///     "result": {
///         "event": "_event_id_"
///     }
/// }
///
/// Lower RPC Format:
/// jsonrpc = {
///     "jsonrpc": "2.0",
///     "id": "0",
///     "method": "lower",
///     "result": {
///         "event": "_event_id_"
///     }
/// }
///
/// Permission RPC Format:
/// jsonrpc = {
///     "jsonrpc": "2.0",
///     "id": "0",
///     "method": "permission",
///     "result": {
///         "result": true // true or false
///     }
/// }
///
/// use in rpc session and rpc
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Response {
    Local(GroupID, RPCParams),
    Upper(GroupID, Option<EventID>),
    Lower(GroupID, Option<EventID>),
    Permission(GroupID, bool),
    Invalid,
}

impl Response {
    pub fn parse(method: &String, params: &RPCParams) -> Self {
        match method.as_str() {
            "local" => {
                let group = Default::default();
                Response::Local(group, params.clone())
            }
            "upper" => {
                let _event_id = params.get("event");
                let event_id = None;
                let group = Default::default();
                Response::Upper(group, event_id)
            }
            "lower" => {
                let _event_id = params.get("event");
                let event_id = None;
                let group = Default::default();
                Response::Lower(group, event_id)
            }
            "permission" => {
                let _permission = params.get("result");
                let permission = true;
                let group = Default::default();
                Response::Permission(group, permission)
            }
            _ => Response::Invalid,
        }
    }

    pub fn deparse(&self) -> (String, RPCParams) {
        match self {
            Response::Local(group, params) => (
                "local".to_owned(),
                json!({
                    "group": &format!("{}", group),
                    "result": params,
                }),
            ),
            Response::Upper(group, event_id) => (
                "upper".to_owned(),
                json!({
                    "group": &format!("{}", group),
                    "result": event_id,
                }),
            ),
            Response::Lower(group, event_id) => (
                "lower".to_owned(),
                json!({
                    "group": &format!("{}", group),
                    "result": event_id,
                }),
            ),
            Response::Permission(group, permission) => (
                "upper".to_owned(),
                json!({
                    "group": &format!("{}", group),
                    "result": permission,
                }),
            ),
            _ => ("invalid".into(), Default::default()),
        }
    }
}
