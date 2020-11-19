mod http;
mod ws;

use smol::{
    channel::{self, Receiver, Sender},
    future,
    net::TcpListener,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;

use tdn_types::{
    message::{RpcMessage as RpcMessageTrait, RpcSendMessage},
    primitive::Result,
    rpc::RpcParam,
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

enum FutureResult {
    Out(RpcSendMessage),
    Stream(RpcMessage),
}

async fn listen<M: 'static + RpcMessageTrait>(
    send: Sender<M>,
    out_recv: Receiver<RpcSendMessage>,
    self_recv: Receiver<RpcMessage>,
) -> Result<()> {
    smol::spawn(async move {
        let mut connections: HashMap<u64, Sender<RpcMessage>> = HashMap::new();

        loop {
            match future::race(
                async { out_recv.recv().await.map(|msg| FutureResult::Out(msg)) },
                async { self_recv.recv().await.map(|msg| FutureResult::Stream(msg)) },
            )
            .await
            {
                Ok(FutureResult::Out(msg)) => {
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
                }
                Ok(FutureResult::Stream(msg)) => {
                    match msg {
                        RpcMessage::Request(id, params, sender) => {
                            let is_ws = sender.is_none();
                            if !is_ws {
                                connections.insert(id, sender.unwrap());
                            }
                            send.send(M::new_rpc(id, params, is_ws))
                                .await
                                .expect("Rpc to Outside channel closed");
                        }
                        RpcMessage::Open(id, sender) => {
                            connections.insert(id, sender);
                        }
                        RpcMessage::Close(id) => {
                            connections.remove(&id);
                        }
                        _ => {} // others not handle
                    }
                }
                Err(_) => break,
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
        TcpListener::bind(config.addr).await.map_err(|e| {
            error!("RPC HTTP listen {:?}", e);
            std::io::Error::new(std::io::ErrorKind::Other, "TCP Listen")
        })?,
    ))
    .detach();

    // ws
    if config.ws.is_some() {
        smol::spawn(ws::ws_listen(
            send,
            TcpListener::bind(config.ws.unwrap()).await.map_err(|e| {
                error!("RPC WS listen {:?}", e);
                std::io::Error::new(std::io::ErrorKind::Other, "TCP Listen")
            })?,
        ))
        .detach();
    }

    Ok(())
}
