use std::net::SocketAddr;
use async_std::sync::{Sender, Receiver};
use async_std::io::Result;
use async_std::task;

use crate::group::GroupId;
use crate::{Message, new_channel};

pub struct LayerConfig {
    pub addr: SocketAddr,
    pub white_list: Vec<SocketAddr>,
    pub black_list: Vec<SocketAddr>,
    pub white_group_list: Vec<GroupId>,
    pub black_group_list: Vec<GroupId>
}

impl LayerConfig {
    pub fn default(addr: SocketAddr) -> Self {
        Self {
            addr: addr,
            white_list: vec![],
            black_list: vec![],
            white_group_list: vec![],
            black_group_list: vec![],
        }
    }
}

pub(crate) struct LayerServer {
    out_send: Sender<Message>,
    config: LayerConfig
}

impl LayerServer {
    pub fn new(out_send: Sender<Message>, config: LayerConfig) -> Self {
        // TODO set layer config

        Self { out_send, config }
    }

    pub async fn start(&mut self) -> Result<Sender<Message>> {
        let (send, recv) = new_channel();

        // start cap rpc server

        // start listen self recv

        Ok(send)
    }
}
