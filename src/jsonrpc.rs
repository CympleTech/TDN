use async_std::{
    io::Result,
    net::{TcpListener, TcpStream},
    sync::{channel, Arc, Receiver, Sender},
    task,
};
use async_tungstenite::{accept_async, tungstenite::protocol::Message as WsMessage};
use futures::{future::LocalBoxFuture, select, sink::SinkExt, FutureExt, StreamExt};
use rand::prelude::*;
use serde_json::json;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::path::PathBuf;

use crate::message::{RpcMessage as RpcMessageTrait, RpcSendMessage};
use crate::primitive::{RpcParam, MAX_MESSAGE_CAPACITY};
use crate::storage::read_string_absolute_file;

pub struct RpcConfig {
    pub addr: SocketAddr,
    pub ws: Option<SocketAddr>,
    pub index: Option<PathBuf>,
}

enum RpcMessage {
    Open(u64, Sender<RpcMessage>),
    Close(u64),
    Request(u64, RpcParam, Option<Sender<RpcMessage>>),
    Response(RpcParam),
}

fn rpc_channel() -> (Sender<RpcMessage>, Receiver<RpcMessage>) {
    channel(MAX_MESSAGE_CAPACITY)
}

fn rpc_send_channel() -> (Sender<RpcSendMessage>, Receiver<RpcSendMessage>) {
    channel(MAX_MESSAGE_CAPACITY)
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
    task::spawn(async move {
        let mut connections: HashMap<u64, Sender<RpcMessage>> = HashMap::new();

        loop {
            select! {
                msg = out_recv.recv().fuse() => match msg {
                    Some(msg) => {
                        let RpcSendMessage(id, params, is_ws) = msg;
                        if is_ws {
                            let s = connections.get(&id);
                            if s.is_some() {
                                s.unwrap().send(RpcMessage::Response(params)).await;
                            }
                        } else {
                            let s = connections.remove(&id);
                            if s.is_some() {
                                s.unwrap().send(RpcMessage::Response(params)).await;
                            }
                        }
                    },
                    None => break,
                },
                msg = self_recv.recv().fuse() => match msg {
                    Some(msg) => {
                        match msg {
                            RpcMessage::Request(id, params, sender) => {
                                let is_ws = sender.is_none();
                                if !is_ws {
                                    connections.insert(id, sender.unwrap());
                                }
                                send.send(M::new_rpc(id, params, is_ws)).await;
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
                    None => break,
                }
            }
        }
    });
    Ok(())
}

struct State {
    send: Arc<(Sender<RpcMessage>, Option<PathBuf>)>,
}

async fn server(send: Sender<RpcMessage>, config: RpcConfig) -> Result<()> {
    let state = State {
        send: Arc::new((send.clone(), config.index)),
    };

    let mut app = tide::with_state(state);

    app.at("/").get(|req: tide::Request<State>| async move {
        let index_body = if req.state().send.1.is_some() {
            let path = req.state().send.1.clone().unwrap();
            read_string_absolute_file(&path).await.ok()
        } else {
            None
        };

        if index_body.is_some() {
            tide::Response::new(200)
                .body_string(index_body.unwrap())
                .set_mime(mime::TEXT_HTML)
        } else {
            tide::Response::new(404)
                .body_string("Not Found Index Page. --- Power By TDN".to_owned())
        }
    });

    app.at("/").post(|mut req: tide::Request<State>| {
        async move {
            let body: String = req.body_string().await.unwrap();
            let res = tide::Response::new(200);

            match parse_jsonrpc(body) {
                Ok((rpc_param, id)) => {
                    let (s_send, s_recv) = rpc_channel();
                    let sender = req.state().send.0.clone();
                    sender
                        .send(RpcMessage::Request(id, rpc_param, Some(s_send.clone())))
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

    // ws
    if config.ws.is_some() {
        task::spawn(ws_listen(
            send,
            TcpListener::bind(config.ws.unwrap()).await?,
        ));
    }

    Ok(())
}

async fn ws_listen(send: Sender<RpcMessage>, listener: TcpListener) -> Result<()> {
    while let Ok((stream, addr)) = listener.accept().await {
        task::spawn(ws_connection(send.clone(), stream, addr));
    }

    Ok(())
}

async fn ws_connection(
    send: Sender<RpcMessage>,
    raw_stream: TcpStream,
    addr: SocketAddr,
) -> Result<()> {
    let ws_stream = accept_async(raw_stream)
        .await
        .map_err(|_e| Error::new(ErrorKind::Other, "Accept WebSocket Failure!"))?;
    println!("DEBUG: WebSocket connection established: {}", addr);
    let id: u64 = rand::thread_rng().gen();
    let (s_send, mut s_recv) = rpc_channel();
    send.send(RpcMessage::Open(id, s_send)).await;

    let (mut writer, mut reader) = ws_stream.split();

    loop {
        select! {
            msg = reader.next().fuse() => match msg {
                Some(msg) => {
                    if msg.is_ok() {
                        let msg = msg.unwrap();
                        let msg = msg.to_text().unwrap();
                        match parse_jsonrpc(msg.to_owned()) {
                            Ok((rpc_param, _id)) => {
                                send.send(RpcMessage::Request(id, rpc_param, None)).await;
                            }
                            Err((err, id)) => {
                                let s = WsMessage::from(err.json(id).to_string());
                                let _ = writer.send(s).await;
                            }
                        }
                    }
                }
                None => break,
            },
            msg = s_recv.next().fuse() => match msg {
                Some(msg) => {
                    let param = match msg {
                        RpcMessage::Response(param) => param,
                        _ => Default::default(),
                    };
                    let s = WsMessage::from(param.to_string());
                    let _ = writer.send(s).await;
                }
                None => break,
            }
        }
    }

    send.send(RpcMessage::Close(id)).await;
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
pub struct RpcHandler<S: 'static + Send + Sync> {
    state: Arc<S>,
    fns: HashMap<String, Box<dyn Fn(Vec<RpcParam>, Arc<S>) -> RpcFut>>,
}

type RpcResult = std::result::Result<RpcParam, RpcError>;
type RpcFut = LocalBoxFuture<'static, RpcResult>;

impl<S: 'static + Send + Sync> RpcHandler<S> {
    pub fn new(state: S) -> RpcHandler<S> {
        Self {
            state: Arc::new(state),
            fns: HashMap::new(),
        }
    }

    pub fn add_method<F: 'static + Fn(Vec<RpcParam>, Arc<S>) -> RpcFut>(
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
                            "result": params,
                        }),
                        Err(err) => err.json(id),
                    }
                }
                None => RpcError::MethodNotFound(method.to_owned()).json(id),
            }
        } else {
            RpcError::InvalidRequest.json(id)
        }
    }
}
