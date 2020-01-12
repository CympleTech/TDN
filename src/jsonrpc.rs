use async_std::{
    io::Result,
    prelude::*,
    sync::{channel, Arc, Receiver, Sender},
    task,
};
use futures::future::LocalBoxFuture;
use futures::{select, FutureExt};
use serde_json::json;
use std::collections::HashMap;
use std::net::SocketAddr;

use crate::primitive::{RpcParam, MAX_MESSAGE_CAPACITY};
use crate::{new_channel, Message};

pub struct RpcConfig {
    pub addr: SocketAddr,
}

enum RpcMessage {
    Request(u64, RpcParam, Sender<RpcMessage>),
    Response(RpcParam),
}

fn rpc_channel() -> (Sender<RpcMessage>, Receiver<RpcMessage>) {
    channel(MAX_MESSAGE_CAPACITY)
}

pub(crate) async fn start(config: RpcConfig, send: Sender<Message>) -> Result<Sender<Message>> {
    let (out_send, out_recv) = new_channel();

    let (self_send, self_recv) = rpc_channel();

    server(self_send, config).await?;
    listen(send, out_recv, self_recv).await?;

    Ok(out_send)
}

async fn listen(
    send: Sender<Message>,
    mut out_recv: Receiver<Message>,
    mut self_recv: Receiver<RpcMessage>,
) -> Result<()> {
    task::spawn(async move {
        let mut connections: HashMap<u64, Sender<RpcMessage>> = HashMap::new();

        loop {
            select! {
                msg = out_recv.next().fuse() => match msg {
                    Some(msg) => {
                        match msg {
                            Message::Rpc(id, params) => {
                                let s = connections.remove(&id);
                                if s.is_some() {
                                    s.unwrap().send(RpcMessage::Response(params)).await;
                                }
                            }
                            _ => {} // others not handle
                        }
                    },
                    None => break,
                },
                msg = self_recv.next().fuse() => match msg {
                    Some(msg) => {
                        match msg {
                            RpcMessage::Request(id, params, sender) => {
                                connections.insert(id, sender);
                                send.send(Message::Rpc(id, params)).await;
                            }
                            _ => {} // others not handle
                        }
                    },
                    None => break,
                }
            }
        }
    });
    Ok(())
}

struct State {
    send: Arc<Sender<RpcMessage>>,
}

async fn server(send: Sender<RpcMessage>, config: RpcConfig) -> Result<()> {
    let state = State {
        send: Arc::new(send),
    };
    let mut app = tide::with_state(state);

    app.at("/").post(|mut req: tide::Request<State>| {
        async move {
            let body: String = req.body_string().await.unwrap();
            let res = tide::Response::new(200);

            match parse_jsonrpc(body) {
                Ok((rpc_param, id)) => {
                    let (s_send, s_recv) = rpc_channel();
                    let sender = req.state().send.clone();
                    sender
                        .send(RpcMessage::Request(id, rpc_param, s_send.clone()))
                        .await;
                    drop(sender);

                    // TODO add timeout.
                    match s_recv.recv().await {
                        Some(msg) => {
                            let param = match msg {
                                RpcMessage::Response(param) => param,
                                _ => Default::default(),
                            };
                            res.body_string(param.to_string())
                                .set_mime(mime::APPLICATION_JSON)
                        }
                        None => res
                            .body_string(Default::default())
                            .set_mime(mime::APPLICATION_JSON),
                    }
                }
                Err((err, id)) => res
                    .body_string(err.json(id).to_string())
                    .set_mime(mime::APPLICATION_JSON),
            }
        }
    });

    task::spawn(app.listen(config.addr));

    Ok(())
}

#[derive(Debug, Clone)]
pub enum RpcError {
    ParseError,
    InvalidRequest,
    InvalidVersion,
    InvalidResponse,
    MethodNotFound(String),
}

impl RpcError {
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
        }
    }
}

fn parse_jsonrpc(json_string: String) -> std::result::Result<(RpcParam, u64), (RpcError, u64)> {
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
/// struct State(u32); // Global State share in all rpc request.
///
/// let mut rpc_handler = RpcHandler::new(State(1));
/// rpc_handler.add_method("echo", |params, state| {
///        Box::pin(async move {
///            assert_eq!(1, state.0);
///            Ok(params)
///    })
/// });
///
/// // when match Message
/// match msg {
///     Message::Rpc(uid, params) => Message::Rpc(uid, params) => {
///         send.send(Message::Rpc(uid, rpc_handler.handle(params).await)).await;
///     }
///     _ => {}
/// }
/// ````

type RpcResult = std::result::Result<RpcParam, RpcError>;
type RpcFut = LocalBoxFuture<'static, RpcResult>;

pub struct RpcHandler<S: 'static + Send + Sync> {
    state: Arc<S>,
    fns: HashMap<String, Box<dyn Fn(RpcParam, Arc<S>) -> RpcFut>>,
}

impl<S: 'static + Send + Sync> RpcHandler<S> {
    pub fn new(state: S) -> RpcHandler<S> {
        Self {
            state: Arc::new(state),
            fns: HashMap::new(),
        }
    }

    pub fn add_method<F: 'static + Fn(RpcParam, Arc<S>) -> RpcFut>(&mut self, name: &str, f: F) {
        self.fns.insert(name.to_owned(), Box::new(f));
    }

    pub async fn handle(&self, mut param: RpcParam) -> RpcParam {
        let id = param["id"].take().as_u64().unwrap();
        let method_s = param["method"].take();
        let method = method_s.as_str().unwrap();
        let params = param["params"].take();

        match self.fns.get(method) {
            Some(f) => {
                let res = f(params, self.state.clone()).await;
                match res {
                    Ok(params) => json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": params,
                    }),
                    Err(err) => err.json(id),
                }
            }
            None => RpcError::MethodNotFound(method.to_owned()).json(id),
        }
    }
}
