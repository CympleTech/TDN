mod http;
mod ws;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::{
    net::TcpListener,
    select,
    sync::mpsc::{self, Receiver, Sender},
};

use tdn_types::{
    message::{ReceiveMessage, RpcSendMessage},
    primitives::Result,
    rpc::RpcParam,
};

pub struct RpcConfig {
    pub addr: SocketAddr,
    pub ws: Option<SocketAddr>,
    pub index: Option<PathBuf>,
}

#[derive(Debug)]
pub(crate) enum RpcMessage {
    Open(u64, Sender<RpcMessage>),
    Close(u64),
    Request(u64, RpcParam, Option<Sender<RpcMessage>>),
    Response(RpcParam),
}

fn rpc_channel() -> (Sender<RpcMessage>, Receiver<RpcMessage>) {
    mpsc::channel(128)
}

fn rpc_send_channel() -> (Sender<RpcSendMessage>, Receiver<RpcSendMessage>) {
    mpsc::channel(128)
}

pub(crate) async fn start(
    config: RpcConfig,
    send: Sender<ReceiveMessage>,
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

async fn listen(
    send: Sender<ReceiveMessage>,
    mut out_recv: Receiver<RpcSendMessage>,
    mut self_recv: Receiver<RpcMessage>,
) -> Result<()> {
    tokio::spawn(async move {
        let mut connections: HashMap<u64, (Sender<RpcMessage>, bool)> = HashMap::new();

        loop {
            let res = select! {
                v = async { out_recv.recv().await.map(|msg| FutureResult::Out(msg)) } => v,
                v = async { self_recv.recv().await.map(|msg| FutureResult::Stream(msg)) } => v
            };

            match res {
                Some(FutureResult::Out(msg)) => {
                    let RpcSendMessage(id, params, is_ws) = msg;
                    if is_ws {
                        if id == 0 {
                            // default send to all ws.
                            for (_, (s, iw)) in &connections {
                                if *iw {
                                    let _ = s.send(RpcMessage::Response(params.clone())).await;
                                }
                            }
                        } else {
                            if let Some((s, _)) = connections.get(&id) {
                                let _ = s.send(RpcMessage::Response(params)).await;
                            }
                        }
                    } else {
                        let s = connections.remove(&id);
                        if s.is_some() {
                            let _ = s.unwrap().0.send(RpcMessage::Response(params)).await;
                        }
                    }
                }
                Some(FutureResult::Stream(msg)) => {
                    match msg {
                        RpcMessage::Request(id, params, sender) => {
                            let is_ws = sender.is_none();
                            if !is_ws {
                                connections.insert(id, (sender.unwrap(), false));
                            }
                            send.send(ReceiveMessage::Rpc(id, params, is_ws))
                                .await
                                .expect("Rpc to Outside channel closed");
                        }
                        RpcMessage::Open(id, sender) => {
                            connections.insert(id, (sender, true));
                        }
                        RpcMessage::Close(id) => {
                            connections.remove(&id);
                        }
                        _ => {} // others not handle
                    }
                }
                None => break,
            }
        }
    });

    Ok(())
}

async fn server(send: Sender<RpcMessage>, config: RpcConfig) -> Result<()> {
    tokio::spawn(http::http_listen(
        config.index.clone(),
        send.clone(),
        TcpListener::bind(config.addr).await.map_err(|e| {
            error!("RPC HTTP listen {:?}", e);
            std::io::Error::new(std::io::ErrorKind::Other, "TCP Listen")
        })?,
    ));

    // ws
    if config.ws.is_some() {
        tokio::spawn(ws::ws_listen(
            send,
            TcpListener::bind(config.ws.unwrap()).await.map_err(|e| {
                error!("RPC WS listen {:?}", e);
                std::io::Error::new(std::io::ErrorKind::Other, "TCP Listen")
            })?,
        ));
    }

    Ok(())
}
