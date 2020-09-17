use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub use serde_json::json;
pub type RpcParam = Value;

use crate::primitive::{HandleResult, Result};

#[derive(Debug, Clone)]
pub enum RpcError<'a> {
    ParseError,
    InvalidRequest,
    InvalidVersion,
    InvalidResponse,
    MethodNotFound(&'a str),
    Custom(&'a str),
}

impl<'a> Into<std::io::Error> for RpcError<'a> {
    fn into(self) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, "RPC Error")
    }
}

impl<'a> RpcError<'a> {
    pub fn json(&self, id: u64) -> RpcParam {
        match self {
            RpcError::ParseError => json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32700,
                    "message": "Parse error"
                }
            }),
            RpcError::MethodNotFound(method) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Method {} not found", method)
                }
            }),
            RpcError::InvalidRequest => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32600,
                    "message": "Invalid Request"
                }
            }),
            RpcError::InvalidVersion => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32600,
                    "message": "Unsupported JSON-RPC protocol version"
                }
            }),
            RpcError::InvalidResponse => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32600,
                    "message": "Invalid Response"
                }
            }),
            RpcError::Custom(m) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32600,
                    "message": m
                }
            }),
        }
    }
}

pub fn parse_jsonrpc<'a>(
    json_string: String,
) -> std::result::Result<RpcParam, (RpcError<'a>, u64)> {
    match serde_json::from_str::<RpcParam>(&json_string) {
        Ok(mut value) => {
            let id_res = value
                .get("id")
                .map(|id| {
                    id.as_u64()
                        .or(id.as_str().map(|sid| sid.parse::<u64>().ok()).flatten())
                })
                .flatten();

            if id_res.is_none() {
                return Err((RpcError::ParseError, 0));
            }
            let id = id_res.unwrap();
            *value.get_mut("id").unwrap() = id.into();

            // check if json is response
            if value.get("result").is_some() || value.get("error").is_some() {
                return Err((RpcError::InvalidResponse, id));
            }

            if value.get("method").is_none() || value.get("method").unwrap().as_str().is_none() {
                return Err((RpcError::InvalidRequest, id));
            }

            if value.get("params").is_none() {
                value["params"] = RpcParam::Array(vec![]);
            }

            let jsonrpc = value
                .get("jsonrpc")
                .map(|v| {
                    v.as_str()
                        .map(|s| if s == "2.0" { Some(2) } else { None })
                        .flatten()
                })
                .flatten();

            if jsonrpc.is_none() {
                return Err((RpcError::InvalidVersion, id));
            }

            Ok(value)
        }
        Err(_e) => Err((RpcError::ParseError, 0)),
    }
}

/// Helpe better handle rpc. Example.
/// ``` rust
/// use tdn_types::rpc::{RpcParam, RpcHandler, json};
///
/// struct State(u32); // Global State share in all rpc request.
///
/// let mut rpc_handler = RpcHandler::new(State(1));
/// rpc_handler.add_method("echo", |params, state|
///     async move {
///         assert_eq!(1, state.0);
///         Ok(RpcParam::Array(params))
///    }
/// );
///
/// let params = json!({"method": "echo", "params": [""]});
/// async {
///     rpc_handler.handle(params).await;
/// };
///
/// // when match Message
/// //match msg {
/// //    Message::Rpc(uid, params) => Message::Rpc(uid, params) => {
/// //        send.send(Message::Rpc(uid, rpc_handler.handle(params).await)).await;
/// //    }
/// //    _ => {}
/// //}
/// ````
pub struct RpcHandler<S: 'static + Send + Sync> {
    state: Arc<S>,
    fns: HashMap<&'static str, Box<DynFutFn<S>>>,
    //fns: HashMap<&'static str, BoxFuture<'static, RpcResult<'static>>>,
}

type RpcResult<'a> = std::result::Result<HandleResult, RpcError<'a>>;
type BoxFuture<'a, RpcResult> = Pin<Box<dyn Future<Output = RpcResult> + Send + 'a>>;

pub trait FutFn<S>: Send + Sync + 'static {
    fn call<'a>(&'a self, params: Vec<RpcParam>, s: Arc<S>) -> BoxFuture<'a, RpcResult<'a>>;
}

pub(crate) type DynFutFn<S> = dyn FutFn<S>;

impl<S, F: Send + Sync + 'static, Fut> FutFn<S> for F
where
    F: Fn(Vec<RpcParam>, Arc<S>) -> Fut,
    Fut: Future<Output = RpcResult<'static>> + Send + 'static,
{
    fn call<'a>(&'a self, params: Vec<RpcParam>, s: Arc<S>) -> BoxFuture<'a, RpcResult<'a>> {
        let fut = (self)(params, s);
        Box::pin(async move { fut.await })
    }
}

impl<S: 'static + Send + Sync> RpcHandler<S> {
    pub fn new(state: S) -> RpcHandler<S> {
        Self {
            state: Arc::new(state),
            fns: HashMap::new(),
        }
    }

    pub fn add_method(&mut self, name: &'static str, f: impl FutFn<S>) {
        self.fns.insert(name, Box::new(f));
    }

    pub async fn handle(&self, mut param: RpcParam) -> Result<HandleResult> {
        let id = param["id"].take().as_u64().unwrap();
        let method_s = param["method"].take();
        let method = method_s.as_str().unwrap();
        let mut new_results = HandleResult::new();

        if let RpcParam::Array(params) = param["params"].take() {
            match self.fns.get(method) {
                Some(f) => {
                    let res = f.call(params, self.state.clone()).await;
                    match res {
                        Ok(HandleResult {
                            rpcs,
                            groups,
                            layers,
                        }) => {
                            new_results.groups = groups;
                            new_results.layers = layers;

                            for params in rpcs {
                                new_results.rpcs.push(rpc_response(id, method, params));
                            }
                        }
                        Err(err) => {
                            let mut res = err.json(id);
                            res["method"] = method.into();
                            new_results.rpcs.push(res);
                        }
                    }
                }
                None => new_results
                    .rpcs
                    .push(RpcError::MethodNotFound(method).json(id)),
            }
        } else {
            new_results.rpcs.push(RpcError::InvalidRequest.json(id))
        }

        Ok(new_results)
    }
}

pub fn rpc_response(id: u64, method: &str, params: RpcParam) -> RpcParam {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "result": params
    })
}
