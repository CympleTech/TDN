use async_std::io::Result;
use async_std::sync::{Receiver, Sender};
use async_std::task;
use std::net::SocketAddr;

use crate::{new_channel, Message};

pub struct RpcConfig {
    pub addr: SocketAddr,
}

pub(crate) async fn start(config: RpcConfig, send: Sender<Message>) -> Result<Sender<Message>> {
    let (out_send, out_recv) = new_channel();

    // start json rpc server

    // start listen self recv

    Ok(out_send)
}
