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
}

use config::Config;
use jsonrpc::start as rpc_start;
use layer::start as layer_start;
use p2p::start as p2p_start;
use primitive::{GroupId, PeerAddr, RpcParam, MAX_MESSAGE_CAPACITY};

/// Group Message Type.
#[derive(Debug)]
pub enum GroupMessage {
    /// peer join, peer_id, peer_socketaddr, join info bytes.
    PeerJoin(PeerAddr, SocketAddr, Vec<u8>),
    /// peer join result. peer_id, is_joined, is_force_close, join info bytes.
    PeerJoinResult(PeerAddr, bool, bool, Vec<u8>),
    /// peer leave. peer_id.
    PeerLeave(PeerAddr),
    /// event between peers. peer_id, bytes.
    Event(PeerAddr, Vec<u8>),
}

/// Layer Message Type.
#[derive(Debug)]
pub enum LayerMessage {
    /// Upper layer send to here, and return send to upper.
    Upper(GroupId, Vec<u8>),
    /// Lower layer send to here, and return send to lower.
    Lower(GroupId, Vec<u8>),
    /// start a upper layer service in layer listen. outside -> tdn.
    UpperJoin(GroupId),
    /// start a upper layer result. tdn -> outside
    UpperJoinResult(GroupId, bool),
    /// remove a upper layer service in layer listen. outside -> tdn.
    UpperLeave(GroupId),
    /// remove a upper layer result. tdn -> outside.
    UpperLeaveResult(GroupId, bool),
    /// request for link to a upper service, and as a lower.
    /// (request_group, remote_group, uuid, addr, data).
    LowerJoin(GroupId, GroupId, u32, SocketAddr, Vec<u8>),
    /// request a upper result.
    /// (request_group, remote_group, uuid, result).
    LowerJoinResult(GroupId, GroupId, u32, bool),
}

/// Message between the server and outside.
#[derive(Debug)]
pub enum Message {
    /// Group: GroupMessage.
    Group(GroupMessage),
    /// Layer: LayerMessage.
    Layer(LayerMessage),
    /// RPC: connection uid, request params, is websocket.
    Rpc(u64, RpcParam, bool),
}

/// new a channel, with Message Type. default capacity is 100.
pub fn new_channel() -> (Sender<Message>, Receiver<Message>) {
    channel::<Message>(MAX_MESSAGE_CAPACITY)
}

/// start multiple services together.
pub async fn multiple_start(
    groups: Vec<(Config, Sender<Message>)>,
) -> Result<HashMap<GroupId, (PeerAddr, Sender<Message>)>> {
    let mut result = HashMap::new();
    for (config, out_send) in groups {
        let (send, recv) = new_channel();
        let gid = config.group_id;
        let peer_addr = start_main(gid, out_send, recv, config).await?;
        result.insert(gid, (peer_addr, send));
    }
    Ok(result)
}

/// start a service, use config.toml file.
/// send a Sender<Message>, and return the peer_id, and service Sender<Message>.
pub async fn start(out_send: Sender<Message>) -> Result<(PeerAddr, Sender<Message>)> {
    let (send, recv) = new_channel();

    let config = Config::load();

    let peer_addr = start_main(config.group_id, out_send, recv, config).await?;

    Ok((peer_addr, send))
}

/// start a service with config.
pub async fn start_with_config(
    out_send: Sender<Message>,
    config: Config,
) -> Result<(PeerAddr, Sender<Message>)> {
    let (send, recv) = new_channel();

    let peer_addr = start_main(config.group_id, out_send, recv, config).await?;

    Ok((peer_addr, send))
}

async fn start_main(
    gid: GroupId,
    out_send: Sender<Message>,
    self_recv: Receiver<Message>,
    config: Config,
) -> Result<PeerAddr> {
    let (p2p_config, layer_config, rpc_config) = config.split();

    // start p2p
    // start layer_rpc
    // start inner json_rpc
    let (p2p_sender_result, layer_sender_result, rpc_sender_result) = join!(
        p2p_start(p2p_config, out_send.clone()),
        layer_start(gid, layer_config, out_send.clone()),
        rpc_start(rpc_config, out_send)
    );
    let ((peer_addr, p2p_sender), layer_sender, rpc_sender) =
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

    Ok(peer_addr)
}
