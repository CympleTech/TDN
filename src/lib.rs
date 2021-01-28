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

// check features conflict
#[cfg(any(
    all(
        feature = "std",
        any(feature = "single", feature = "multiple", feature = "full")
    ),
    all(
        feature = "single",
        any(feature = "std", feature = "multiple", feature = "full")
    ),
    all(
        feature = "multiple",
        any(feature = "std", feature = "single", feature = "full")
    ),
    all(
        feature = "full",
        any(feature = "std", feature = "single", feature = "multiple")
    ),
))]
panic!("feature conflict, only one feature at one time.");

#[macro_use]
extern crate log;

mod config;
mod group;
mod rpc;

#[cfg(any(feature = "std", feature = "full"))]
mod layer;

// public mod
pub mod error;

// re-export smol
pub use smol;
// re-export tdn_types
pub use tdn_types as types;

// public struct
pub mod prelude {
    pub use super::config::Config;

    pub use tdn_types::group::GroupId;
    pub use tdn_types::message::{
        GroupReceiveMessage, GroupSendMessage, StateRequest, StateResponse,
    };

    #[cfg(any(feature = "std", feature = "full"))]
    pub use tdn_types::message::{LayerReceiveMessage, LayerSendMessage};

    pub use tdn_types::message::{ReceiveMessage, SendMessage};
    pub use tdn_types::primitive::{Broadcast, HandleResult, PeerAddr};

    use chamomile::prelude::{start as chamomile_start, ReceiveMessage as ChamomileReceiveMessage};
    use smol::{
        channel::{self, Receiver, Sender},
        future,
        io::Result,
    };
    use tdn_types::message::RpcSendMessage;

    use super::group::*;
    use super::rpc::start as rpc_start;

    #[cfg(any(feature = "std", feature = "full"))]
    use super::layer::*;

    /// new a channel, send message to TDN Message. default capacity is 1024.
    pub fn new_send_channel() -> (Sender<SendMessage>, Receiver<SendMessage>) {
        channel::unbounded()
    }

    /// new a channel, send message to TDN Message. default capacity is 1024.
    pub fn new_receive_channel() -> (Sender<ReceiveMessage>, Receiver<ReceiveMessage>) {
        channel::unbounded()
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
        _gid: GroupId,
        out_send: Sender<ReceiveMessage>,
        self_recv: Receiver<SendMessage>,
        config: Config,
    ) -> Result<PeerAddr> {
        let (p2p_config, rpc_config) = config.split();

        // start chamomile network & inner rpc.
        let ((peer_id, p2p_send, p2p_recv), rpc_sender) = future::try_zip(
            chamomile_start(p2p_config),
            rpc_start(rpc_config, out_send.clone()),
        )
        .await?;

        debug!("chamomile & jsonrpc service started");

        // handle outside msg.
        smol::spawn(async move {
            let my_groups: Vec<GroupId> = Vec::new();

            while let Ok(message) = self_recv.recv().await {
                match message {
                    #[cfg(any(feature = "single", feature = "std"))]
                    SendMessage::Group(msg) => {
                        group_handle_send(&my_groups[0], &p2p_send, msg)
                            .await
                            .map_err(|e| error!("Chamomile channel: {:?}", e))
                            .expect("Chamomile channel closed");
                    }
                    #[cfg(any(feature = "multiple", feature = "full"))]
                    SendMessage::Group(group_id, msg) => {
                        group_handle_send(&group_id, &p2p_send, msg)
                            .await
                            .map_err(|e| error!("Chamomile channel: {:?}", e))
                            .expect("Chamomile channel closed");
                    }
                    SendMessage::Rpc(uid, param, is_ws) => {
                        rpc_sender
                            .send(RpcSendMessage(uid, param, is_ws))
                            .await
                            .map_err(|e| error!("Rpc channel: {:?}", e))
                            .expect("Rpc channel closed");
                    }
                    #[cfg(feature = "std")]
                    SendMessage::Layer(tgid, msg) => {
                        layer_handle_send(&my_groups[0], tgid, &p2p_send, msg)
                            .await
                            .map_err(|e| error!("Chamomile channel: {:?}", e))
                            .expect("Chamomile channel closed");
                    }
                    #[cfg(feature = "full")]
                    SendMessage::Layer(fgid, tgid, msg) => {
                        layer_handle_send(&fgid, tgid, &p2p_send, msg)
                            .await
                            .map_err(|e| error!("Chamomile channel: {:?}", e))
                            .expect("Chamomile channel closed");
                    }
                }
            }
        })
        .detach();

        // handle chamomile send msg.
        smol::spawn(async move {
            let my_groups: Vec<GroupId> = Vec::new();

            // if group's inner message, from_group in our groups.
            // if layer's message,       from_group not in our groups.

            while let Ok(message) = p2p_recv.recv().await {
                match message {
                    ChamomileReceiveMessage::StableConnect(peer_addr, data) => {
                        let gid = Default::default();

                        if my_groups.contains(&gid) {
                            let _ =
                                group_handle_recv_stable_connect(&gid, &out_send, peer_addr, data)
                                    .await;
                        } else {
                            // layer handle it.
                            let _ =
                                layer_handle_recv_connect(gid, &out_send, peer_addr, data).await;
                        }
                    }
                    ChamomileReceiveMessage::StableResult(peer_addr, is_ok, data) => {
                        let gid = Default::default();

                        if my_groups.contains(&gid) {
                            let _ = group_handle_recv_stable_result(
                                &gid, &out_send, peer_addr, is_ok, data,
                            )
                            .await;
                        } else {
                            // layer handle it.
                            let _ =
                                layer_handle_recv_result(gid, &out_send, peer_addr, is_ok, data)
                                    .await;
                        }
                    }
                    ChamomileReceiveMessage::StableLeave(peer_addr) => {
                        for gid in &my_groups {
                            let _ =
                                group_handle_recv_stable_leave(&gid, &out_send, peer_addr).await;
                            let _ = layer_handle_recv_leave(*gid, &out_send, peer_addr).await;
                        }

                        // layer handle it.
                    }
                    ChamomileReceiveMessage::Data(peer_addr, data) => {
                        let gid = Default::default();

                        if my_groups.contains(&gid) {
                            let _ = group_handle_recv_data(&gid, &out_send, peer_addr, data).await;
                        } else {
                            // layer handle it.
                            let _ = layer_handle_recv_data(gid, &out_send, peer_addr, data).await;
                        }
                    }
                    ChamomileReceiveMessage::Stream(id, stream) => {
                        let gid = Default::default();

                        if my_groups.contains(&gid) {
                            let _ = group_handle_recv_stream(&gid, &out_send, id, stream).await;
                        } else {
                            // layer handle it.
                            let _ = layer_handle_recv_stream(gid, &out_send, id, stream).await;
                        }
                    }
                    ChamomileReceiveMessage::Delivery(t, tid, is_ok) => {
                        let gid = Default::default();

                        if my_groups.contains(&gid) {
                            let _ =
                                group_handle_recv_delivery(&gid, &out_send, t.into(), tid, is_ok)
                                    .await;
                        } else {
                            // layer handle it.
                            let _ =
                                layer_handle_recv_delivery(gid, &out_send, t.into(), tid, is_ok)
                                    .await;
                        }
                    }
                }
            }
        })
        .detach();

        Ok(peer_id)
    }
}
