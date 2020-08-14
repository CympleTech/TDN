//! `TDN` - Trusted Distributed Network.
//!
//! Blockchain infrastructure framework for security and trusted distributed interactive.
//!
//! TDN is underlying network (including p2p, rpc, and other special transports)
//! and application framework built on Groups and Layers, we built this framework
//! because we feel that the blockchain is very limited. If you want a more open
//! and free distributed application development technology, and Pluggable,
//! lightweight application framework, TDN can satisfy you.

#![recursion_limit = "1024"]
#![feature(associated_type_defaults)]

#[macro_use]
extern crate log;

mod config;
mod layer;
mod message;
mod p2p;
mod rpc;

// public mod
pub use async_std; // pub async std to others
pub mod error;
pub mod primitive;
pub mod storage;
pub mod traits;

// public struct
pub mod prelude {
    pub use super::config::Config;
    pub use super::message::{GroupReceiveMessage, GroupSendMessage};
    pub use super::message::{LayerReceiveMessage, LayerSendMessage};
    pub use super::message::{ReceiveMessage, SendMessage};
    pub use super::message::{SingleReceiveMessage, SingleSendMessage};
    pub use super::primitive::{Broadcast, GroupId, PeerAddr, RpcParam};
    pub use super::rpc::{RpcError, RpcHandler};

    use async_std::{
        io::Result,
        sync::{channel, Receiver, Sender},
        task,
    };
    use futures::join;
    use std::collections::HashMap;

    use super::layer::start as layer_start;
    use super::message::RpcSendMessage;
    use super::p2p::start as p2p_start;
    use super::primitive::MAX_MESSAGE_CAPACITY;
    use super::rpc::start as rpc_start;

    /// new a channel, send message to TDN Message. default capacity is 1024.
    pub fn new_send_channel() -> (Sender<SendMessage>, Receiver<SendMessage>) {
        channel(MAX_MESSAGE_CAPACITY)
    }

    /// new a channel, send message to TDN Message. default capacity is 1024.
    pub fn new_receive_channel() -> (Sender<ReceiveMessage>, Receiver<ReceiveMessage>) {
        channel(MAX_MESSAGE_CAPACITY)
    }

    /// new a signle layer channel, send message to TDN. default capacity is 1024.
    pub fn new_single_send_channel() -> (Sender<SingleSendMessage>, Receiver<SingleSendMessage>) {
        channel(MAX_MESSAGE_CAPACITY)
    }

    /// new a signle layer channel, receive message from TDN. default capacity is 1024.
    pub fn new_single_receive_channel(
    ) -> (Sender<SingleReceiveMessage>, Receiver<SingleReceiveMessage>) {
        channel(MAX_MESSAGE_CAPACITY)
    }

    /// start multiple services together.
    pub async fn multiple_start(
        groups: Vec<Config>,
    ) -> Result<HashMap<GroupId, (PeerAddr, Sender<SendMessage>, Receiver<ReceiveMessage>)>> {
        let mut result = HashMap::new();
        for config in groups {
            let (send_send, send_recv) = new_send_channel();
            let (recv_send, recv_recv) = new_receive_channel();
            let gid = config.group_id;
            let peer_addr = start_main(gid, recv_send, send_recv, config).await?;
            result.insert(gid, (peer_addr, send_send, recv_recv));
        }
        Ok(result)
    }

    /// start a service, use config.toml file.
    /// send a Sender<Message>, and return the peer_id, and service Sender<Message>.
    pub async fn start() -> Result<(PeerAddr, Sender<SendMessage>, Receiver<ReceiveMessage>)> {
        let (send_send, send_recv) = new_send_channel();
        let (recv_send, recv_recv) = new_receive_channel();

        let config = Config::load().await;

        let peer_addr = start_main(config.group_id, recv_send, send_recv, config).await?;

        Ok((peer_addr, send_send, recv_recv))
    }

    /// start a service with config.
    pub async fn start_with_config(
        config: Config,
    ) -> Result<(PeerAddr, Sender<SendMessage>, Receiver<ReceiveMessage>)> {
        let (send_send, send_recv) = new_send_channel();
        let (recv_send, recv_recv) = new_receive_channel();

        let peer_addr = start_main(config.group_id, recv_send, send_recv, config).await?;

        Ok((peer_addr, send_send, recv_recv))
    }

    async fn start_main(
        gid: GroupId,
        out_send: Sender<ReceiveMessage>,
        self_recv: Receiver<SendMessage>,
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
            while let Ok(message) = self_recv.recv().await {
                match message {
                    SendMessage::Layer(msg) => layer_sender.send(msg).await,
                    SendMessage::Group(msg) => p2p_sender.send(msg).await,
                    SendMessage::Rpc(uid, param, is_ws) => {
                        rpc_sender.send(RpcSendMessage(uid, param, is_ws)).await;
                    }
                }
            }
        });

        Ok(peer_addr)
    }

    /// start a signle layer service, use config.toml file.
    /// send a Sender<Message>, and return the peer_id, and service Sender<Message>.
    pub async fn single_start() -> Result<(
        PeerAddr,
        Sender<SingleSendMessage>,
        Receiver<SingleReceiveMessage>,
    )> {
        let (send_send, send_recv) = new_single_send_channel();
        let (recv_send, recv_recv) = new_single_receive_channel();

        let config = Config::load().await;

        let peer_addr = single_start_main(recv_send, send_recv, config).await?;

        Ok((peer_addr, send_send, recv_recv))
    }

    /// start a single layer service with config.
    pub async fn single_start_with_config(
        config: Config,
    ) -> Result<(
        PeerAddr,
        Sender<SingleSendMessage>,
        Receiver<SingleReceiveMessage>,
    )> {
        let (send_send, send_recv) = new_single_send_channel();
        let (recv_send, recv_recv) = new_single_receive_channel();

        let peer_addr = single_start_main(recv_send, send_recv, config).await?;

        Ok((peer_addr, send_send, recv_recv))
    }

    async fn single_start_main(
        out_send: Sender<SingleReceiveMessage>,
        self_recv: Receiver<SingleSendMessage>,
        config: Config,
    ) -> Result<PeerAddr> {
        let (p2p_config, _, rpc_config) = config.split();

        // start p2p
        // start inner json_rpc
        let (p2p_sender_result, rpc_sender_result) = join!(
            p2p_start(p2p_config, out_send.clone()),
            rpc_start(rpc_config, out_send)
        );
        let ((peer_addr, p2p_sender), rpc_sender) = (p2p_sender_result?, rpc_sender_result?);

        task::spawn(async move {
            while let Ok(message) = self_recv.recv().await {
                match message {
                    SingleSendMessage::Group(msg) => p2p_sender.send(msg).await,
                    SingleSendMessage::Rpc(uid, param, is_ws) => {
                        rpc_sender.send(RpcSendMessage(uid, param, is_ws)).await;
                    }
                }
            }
        });

        Ok(peer_addr)
    }
}
