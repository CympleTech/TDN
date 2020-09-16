mod http;
mod ws;

use futures::{future::LocalBoxFuture, select, FutureExt};
use smol::{
    channel::{self, Receiver, Sender},
    io::Result,
    net::TcpListener,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use tdn_types::{
    message::{RpcMessage as RpcMessageTrait, RpcSendMessage},
    primitive::{json, RpcParam},
};

pub struct RpcConfig {
    pub addr: SocketAddr,
    pub ws: Option<SocketAddr>,
    pub index: Option<PathBuf>,
}

pub(crate) enum RpcMessage {
    Open(u64, Sender<RpcMessage>),
    Close(u64),
    Request(u64, RpcParam, Option<Sender<RpcMessage>>),
    Response(RpcParam),
}

fn rpc_channel() -> (Sender<RpcMessage>, Receiver<RpcMessage>) {
    channel::unbounded()
}

fn rpc_send_channel() -> (Sender<RpcSendMessage>, Receiver<RpcSendMessage>) {
    channel::unbounded()
}

pub(crate) async fn start<M: 'static + RpcMessageTrait>(
    config: RpcConfig,
    send: Sender<M>,
) -> Result<Sender<RpcSendMessage>> {
    let (out_send, out_recv) = rpc_send_channel();

    let (self_send, self_recv) = rpc_channel();

    server(self_send, config).await?;
    listen(send, out_recv, self_recv).await?;

    Ok(out_send)
}

async fn listen<M: 'static + RpcMessageTrait>(
    send: Sender<M>,
    out_recv: Receiver<RpcSendMessage>,
    self_recv: Receiver<RpcMessage>,
) -> Result<()> {
    smol::spawn(async move {
        let mut connections: HashMap<u64, Sender<RpcMessage>> = HashMap::new();

        loop {
            select! {
                msg = out_recv.recv().fuse() => match msg {
                    Ok(msg) => {
                        let RpcSendMessage(id, params, is_ws) = msg;
                        if is_ws {
                            let s = connections.get(&id);
                            if s.is_some() {
                                let _ = s.unwrap().send(RpcMessage::Response(params)).await;
                            }
                        } else {
                            let s = connections.remove(&id);
                            if s.is_some() {
                                let _ = s.unwrap().send(RpcMessage::Response(params)).await;
                            }
                        }
                    },
                    Err(_) => break,
                },
                msg = self_recv.recv().fuse() => match msg {
                    Ok(msg) => {
                        match msg {
                            RpcMessage::Request(id, params, sender) => {
                                let is_ws = sender.is_none();
                                if !is_ws {
                                    connections.insert(id, sender.unwrap());
                                }
                                send.send(M::new_rpc(id, params, is_ws))
                                    .await.expect("Rpc to Outside channel closed");
                            }
                            RpcMessage::Open(id, sender) => {
                                connections.insert(id, sender);
                            }
                            RpcMessage::Close(id) => {
                                connections.remove(&id);
                            }
                            _ => {} // others not handle
                        }
                    },
                    Err(_) => break,
                }
            }
        }
    })
    .detach();

    Ok(())
}

async fn server(send: Sender<RpcMessage>, config: RpcConfig) -> Result<()> {
    smol::spawn(http::http_listen(
        config.index.clone(),
        send.clone(),
        TcpListener::bind(config.addr).await?,
    ))
    .detach();

    // ws
    if config.ws.is_some() {
        smol::spawn(ws::ws_listen(
            send,
            TcpListener::bind(config.ws.unwrap()).await?,
        ))
        .detach();
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub enum RpcError<'a> {
    ParseError,
    InvalidRequest,
    InvalidVersion,
    InvalidResponse,
    MethodNotFound(&'a str),
    Custom(&'a str),
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

fn parse_jsonrpc<'a>(
    json_string: String,
) -> std::result::Result<(RpcParam, u64), (RpcError<'a>, u64)> {
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

            Ok((value, rand::random::<u64>()))
        }
        Err(_e) => Err((RpcError::ParseError, 0)),
    }
}

/// Helpe better handle rpc. Example.
/// ``` rust
/// use tdn::prelude::{RpcParam, RpcHandler};
/// use serde_json::json;
///
/// struct State(u32); // Global State share in all rpc request.
///
/// let mut rpc_handler = RpcHandler::new(State(1));
/// rpc_handler.add_method("echo", |params, state| {
///        Box::pin(async move {
///            assert_eq!(1, state.0);
///            Ok(RpcParam::Array(params))
///    })
/// });
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
pub struct RpcHandler<S> {
    state: Arc<S>,
    fns: HashMap<String, Box<dyn Fn(Vec<RpcParam>, Arc<S>) -> RpcFut<'static>>>,
}

type RpcResult<'a> = std::result::Result<RpcParam, RpcError<'a>>;
type RpcFut<'a> = LocalBoxFuture<'static, RpcResult<'a>>;

impl<S> RpcHandler<S> {
    pub fn new(state: S) -> RpcHandler<S> {
        Self {
            state: Arc::new(state),
            fns: HashMap::new(),
        }
    }

    pub fn add_method<F: 'static + Fn(Vec<RpcParam>, Arc<S>) -> RpcFut<'static>>(
        &mut self,
        name: &str,
        f: F,
    ) {
        self.fns.insert(name.to_owned(), Box::new(f));
    }

    pub async fn handle(&self, mut param: RpcParam) -> RpcParam {
        let id = param["id"].take().as_u64().unwrap();
        let method_s = param["method"].take();
        let method = method_s.as_str().unwrap();
        if let RpcParam::Array(params) = param["params"].take() {
            match self.fns.get(method) {
                Some(f) => {
                    let res = f(params, self.state.clone()).await;
                    match res {
                        Ok(params) => json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "method": method,
                            "result": params,
                        }),
                        Err(err) => {
                            let mut res = err.json(id);
                            res["method"] = method.into();
                            res
                        }
                    }
                }
                None => RpcError::MethodNotFound(method).json(id),
            }
        } else {
            RpcError::InvalidRequest.json(id)
        }
    }
}
