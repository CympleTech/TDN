use async_std::{
    io::Result,
    prelude::*,
    sync::{channel, Arc, Receiver, Sender},
    task,
};
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
            let res = tide::Response::new(200).set_mime(mime::APPLICATION_JSON);

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
                        }
                        None => res.body_string(Default::default()),
                    }
                }
                Err(err) => res.body_string(err.json().to_string()),
            }
        }
    });

    task::spawn(app.listen(config.addr));

    Ok(())
}

#[derive(Debug, Clone)]
pub enum RpcError {
    ParseError,
    MethodNotFound(u64, String),
    InvalidRequest(u64),
    InvalidVersion(u64),
    InvalidResponse(u64),
}

impl RpcError {
    pub fn json(&self) -> RpcParam {
        match self {
            RpcError::ParseError => json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32700,
                    "message": "Parse error"
                }
            }),
            RpcError::MethodNotFound(id, method) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Method {} not found", method)
                }
            }),
            RpcError::InvalidRequest(id) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32600,
                    "message": "Invalid Request"
                }
            }),
            RpcError::InvalidVersion(id) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32600,
                    "message": "Unsupported JSON-RPC protocol version"
                }
            }),
            RpcError::InvalidResponse(id) => json!({
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

fn parse_jsonrpc(json_string: String) -> std::result::Result<(RpcParam, u64), RpcError> {
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
                return Err(RpcError::ParseError);
            }
            let id = id_res.unwrap();
            *value.get_mut("id").unwrap() = id.into();

            // check if json is response
            if value.get("result").is_some() || value.get("error").is_some() {
                return Err(RpcError::InvalidResponse(id));
            }

            if value.get("method").is_none() {
                return Err(RpcError::InvalidRequest(id));
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
                return Err(RpcError::InvalidVersion(id));
            }

            Ok((value, rand::random::<u64>()))
        }
        Err(_e) => Err(RpcError::ParseError),
    }
}
