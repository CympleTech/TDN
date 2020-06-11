use async_std::{
    io::Result,
    net::{TcpListener, TcpStream},
    sync::{Arc, RwLock, Sender},
    task,
};
//use async_tungstenite::{accept_async, tungstenite::protocol::Message as WsMessage};
//use futures::{select, sink::SinkExt, FutureExt, StreamExt};
//use rand::prelude::*;
//use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::path::PathBuf;
//use std::time::Duration;

use crate::storage::read_string_absolute_file;

use super::RpcMessage;
//use super::{parse_jsonrpc, rpc_channel, RpcMessage};

pub(crate) async fn http_listen(
    index: Option<PathBuf>,
    send: Sender<RpcMessage>,
    listener: TcpListener,
) -> Result<()> {
    let homepage = if let Some(path) = index {
        read_string_absolute_file(&path)
            .await
            .unwrap_or("Error Homepage.".to_owned())
    } else {
        "No Homepage.".to_owned()
    };
    let homelink = Arc::new(RwLock::new(homepage));

    while let Ok((stream, addr)) = listener.accept().await {
        task::spawn(http_connection(
            homelink.clone(),
            send.clone(),
            stream,
            addr,
        ));
    }

    Ok(())
}

async fn http_connection(
    _homelink: Arc<RwLock<String>>,
    _send: Sender<RpcMessage>,
    raw_stream: TcpStream,
    _addr: SocketAddr,
) -> Result<()> {
    debug!("processing jsonrpc request.");
    let (mut _reader, mut _writer) = &mut (&raw_stream, &raw_stream);

    Ok(())
}
