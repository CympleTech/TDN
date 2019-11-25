use std::net::SocketAddr;
use async_std::sync::{channel, Sender, Receiver};
use async_std::io::Result;
use async_std::task;

use crate::{Message, new_channel};

pub struct RpcConfig {
    pub addr: SocketAddr,
}

impl RpcConfig {
    pub fn default(addr: SocketAddr) -> Self {
        Self { addr }
    }
}

pub(crate) struct JsonRpc {
    config: RpcConfig,
}

impl JsonRpc {
    pub fn new( config: RpcConfig) -> Self {
        // TODO set layer config

        Self { config }
    }

    pub async fn start(server: JsonRpc, out_send: Sender<Message>) -> Result<Sender<Message>> {
        let (send, recv) = new_channel();

        // start json rpc server

        // start listen self recv

        Ok(send)
    }
}
