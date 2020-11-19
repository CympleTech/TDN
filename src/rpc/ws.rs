use async_tungstenite::{accept_async, tungstenite::protocol::Message as WsMessage};
use rand::prelude::*;
use smol::{
    channel::{RecvError, Sender},
    future,
    io::Result,
    net::{TcpListener, TcpStream},
};
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;

use tdn_types::rpc::parse_jsonrpc;

use futures_util::{SinkExt, StreamExt};

use super::{rpc_channel, RpcMessage};

pub(crate) async fn ws_listen(send: Sender<RpcMessage>, listener: TcpListener) -> Result<()> {
    while let Ok((stream, addr)) = listener.accept().await {
        smol::spawn(ws_connection(send.clone(), stream, addr)).detach();
    }

    Ok(())
}

enum FutureResult {
    Out(RpcMessage),
    Stream(WsMessage),
}

async fn ws_connection(
    send: Sender<RpcMessage>,
    raw_stream: TcpStream,
    addr: SocketAddr,
) -> Result<()> {
    let ws_stream = accept_async(raw_stream)
        .await
        .map_err(|_e| Error::new(ErrorKind::Other, "Accept WebSocket Failure!"))?;
    debug!("DEBUG: WebSocket connection established: {}", addr);
    let id: u64 = rand::thread_rng().gen();
    let (s_send, s_recv) = rpc_channel();
    send.send(RpcMessage::Open(id, s_send))
        .await
        .expect("Ws to Rpc channel closed");

    let (mut writer, mut reader) = ws_stream.split();

    loop {
        match future::race(
            async { s_recv.recv().await.map(|msg| FutureResult::Out(msg)) },
            async {
                reader
                    .next()
                    .await
                    .map(|msg| msg.map(|msg| FutureResult::Stream(msg)).ok())
                    .flatten()
                    .ok_or(RecvError)
            },
        )
        .await
        {
            Ok(FutureResult::Out(msg)) => {
                let param = match msg {
                    RpcMessage::Response(param) => param,
                    _ => Default::default(),
                };
                let s = WsMessage::from(param.to_string());
                let _ = writer.send(s).await;
            }
            Ok(FutureResult::Stream(msg)) => {
                let msg = msg.to_text().unwrap();
                match parse_jsonrpc(msg.to_owned()) {
                    Ok(rpc_param) => {
                        send.send(RpcMessage::Request(id, rpc_param, None))
                            .await
                            .expect("Ws to Rpc channel closed");
                    }
                    Err((err, id)) => {
                        let s = WsMessage::from(err.json(id).to_string());
                        let _ = writer.send(s).await;
                    }
                }
            }
            Err(_) => break,
        }
    }

    send.send(RpcMessage::Close(id))
        .await
        .expect("Ws to Rpc channel closed");
    Ok(())
}
