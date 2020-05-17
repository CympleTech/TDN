use async_std::{
    io::Result,
    net::{TcpListener, TcpStream},
    sync::Sender,
    task,
};
use async_tungstenite::{accept_async, tungstenite::protocol::Message as WsMessage};
use futures::{select, sink::SinkExt, FutureExt, StreamExt};
use rand::prelude::*;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;

use super::{parse_jsonrpc, rpc_channel, RpcMessage};

pub(crate) async fn ws_listen(send: Sender<RpcMessage>, listener: TcpListener) -> Result<()> {
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
    debug!("DEBUG: WebSocket connection established: {}", addr);
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
