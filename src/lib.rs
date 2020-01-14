#![recursion_limit = "1024"]
#![feature(associated_type_defaults)]

use async_std::{
    io::Result,
    sync::{channel, Receiver, Sender},
    task,
};
use futures::join;
use std::collections::HashMap;
use std::net::SocketAddr;

mod config;
mod jsonrpc;
mod layer;
mod p2p;

// public mod
pub use async_std; // pub async std to others
pub mod error;
pub mod primitive;
pub mod storage;
pub mod traits;

// public struct
pub mod prelude {
    pub use super::config::Config;
    pub use super::jsonrpc::{RpcError, RpcHandler};
    pub use super::primitive::GroupId;
    pub use super::primitive::PeerAddr;
    pub use super::primitive::RpcParam;
    pub use super::GroupMessage;
    pub use super::LayerMessage;
    pub use super::Message;

    // TODO ADD start function
}

use config::Config;
use jsonrpc::start as rpc_start;
use layer::start as layer_start;
use p2p::start as p2p_start;
use primitive::{GroupId, PeerAddr, RpcParam, MAX_MESSAGE_CAPACITY};

#[derive(Debug)]
pub enum GroupMessage {
    PeerJoin(PeerAddr, SocketAddr, Vec<u8>),
    PeerJoinResult(PeerAddr, bool, Vec<u8>),
    PeerLeave(PeerAddr),
    Event(PeerAddr, Vec<u8>),
}

#[derive(Debug)]
pub enum LayerMessage {
    Upper(GroupId, Vec<u8>),
    Lower(GroupId, Vec<u8>),
    LayerJoin(GroupId, u32, SocketAddr, Vec<u8>),
    LayerJoinResult(GroupId, u32, bool),
}

#[derive(Debug)]
pub enum Message {
    Group(GroupMessage),
    Layer(LayerMessage),
    Rpc(u64, RpcParam, bool),
}

pub fn new_channel() -> (Sender<Message>, Receiver<Message>) {
    channel::<Message>(MAX_MESSAGE_CAPACITY)
}

pub async fn multiple_start(
    groups: Vec<(Config, Sender<Message>)>,
) -> Result<HashMap<GroupId, Sender<Message>>> {
    let mut result = HashMap::new();
    for (config, out_send) in groups {
        let (send, recv) = new_channel();
        let gid = config.group_id;
        start_main(gid, out_send, recv, config).await?;
        result.insert(gid, send);
    }
    Ok(result)
}

pub async fn start(out_send: Sender<Message>) -> Result<Sender<Message>> {
    let (send, recv) = new_channel();

    let config = Config::load();

    start_main(config.group_id, out_send, recv, config).await?;

    Ok(send)
}

pub async fn start_with_config(
    out_send: Sender<Message>,
    config: Config,
) -> Result<Sender<Message>> {
    let (send, recv) = new_channel();

    start_main(config.group_id, out_send, recv, config).await?;

    Ok(send)
}

async fn start_main(
    gid: GroupId,
    out_send: Sender<Message>,
    self_recv: Receiver<Message>,
    config: Config,
) -> Result<()> {
    let (p2p_config, layer_config, rpc_config) = config.split();

    // start p2p
    // start layer_rpc
    // start inner json_rpc
    let (p2p_sender_result, layer_sender_result, rpc_sender_result) = join!(
        p2p_start(gid, p2p_config, out_send.clone()),
        layer_start(gid, layer_config, out_send.clone()),
        rpc_start(rpc_config, out_send)
    );
    let (p2p_sender, layer_sender, rpc_sender) =
        (p2p_sender_result?, layer_sender_result?, rpc_sender_result?);

    task::spawn(async move {
        while let Some(message) = self_recv.recv().await {
            let sender = match message {
                Message::Group { .. } => &p2p_sender,
                Message::Layer { .. } => &layer_sender,
                Message::Rpc { .. } => &rpc_sender,
            };
            sender.send(message).await;
        }
    });

    Ok(())
}
