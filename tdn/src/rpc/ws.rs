use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaChaRng,
};
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use tokio::{
    net::{TcpListener, TcpStream},
    select,
    sync::mpsc::Sender,
};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message as WsMessage};

use tdn_types::{primitives::Result, rpc::parse_jsonrpc};

use futures_util::{SinkExt, StreamExt};

use super::{rpc_channel, RpcMessage};

pub(crate) async fn ws_listen(send: Sender<RpcMessage>, listener: TcpListener) -> Result<()> {
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(ws_connection(send.clone(), stream, addr));
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

    let mut rng = ChaChaRng::from_entropy();
    let id: u64 = rng.next_u64();
    let (s_send, mut s_recv) = rpc_channel();
    send.send(RpcMessage::Open(id, s_send)).await?;

    let (mut writer, mut reader) = ws_stream.split();

    loop {
        let res = select! {
            v = async { s_recv.recv().await.map(|msg| FutureResult::Out(msg)) } => v,
            v = async {
                reader
                    .next()
                    .await
                    .map(|msg| msg.map(|msg| FutureResult::Stream(msg)).ok())
                    .flatten()
            } => v,
        };

        match res {
            Some(FutureResult::Out(msg)) => {
                let param = match msg {
                    RpcMessage::Response(param) => param,
                    _ => Default::default(),
                };
                let s = WsMessage::from(param.to_string());
                let _ = writer.send(s).await;
            }
            Some(FutureResult::Stream(msg)) => {
                let msg = msg.to_text().unwrap();
                if msg == "ping" {
                    let s = WsMessage::from("pong".to_owned());
                    let _ = writer.send(s).await;
                    continue;
                }

                match parse_jsonrpc(msg.to_owned()) {
                    Ok(rpc_param) => {
                        send.send(RpcMessage::Request(id, rpc_param, None)).await?;
                    }
                    Err((err, id)) => {
                        let s = WsMessage::from(err.json(id).to_string());
                        let _ = writer.send(s).await;
                    }
                }
            }
            None => break,
        }
    }

    send.send(RpcMessage::Close(id)).await?;
    Ok(())
}
