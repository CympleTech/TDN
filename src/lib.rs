use async_std::io::Result;
use async_std::sync::{channel, Receiver, Sender};
use async_std::task;
use futures::join;
use std::net::SocketAddr;

mod config;
mod group;
mod jsonrpc;
mod layer;
mod p2p;

use jsonrpc::{start as rpc_start, RpcConfig};
use layer::{start as layer_start, LayerConfig};
use p2p::{start as p2p_start, P2pConfig};

pub use group::{Group, GroupId};
pub use p2p::PeerId;

pub const MAX_MESSAGE_CAPACITY: usize = 1024;

#[derive(Debug)]
pub struct PeerInfo;

#[derive(Debug)]
pub enum Message {
    PeerJoin(GroupId, PeerId, PeerInfo, Option<SocketAddr>),
    PeerLeave(GroupId, PeerId),
    Event(GroupId, PeerId, Vec<u8>),
    Upper(GroupId, PeerId, Vec<u8>),
    Lower(GroupId, PeerId, Vec<u8>),
    Permission(GroupId, PeerId, Option<SocketAddr>),
    Rpc(Vec<u8>),
}

pub struct Config {
    p2p: P2pConfig,
    layer: LayerConfig,
    rpc: RpcConfig,
}

impl Config {
    pub fn default(p2p_addr: SocketAddr, layer_addr: SocketAddr, rpc_addr: SocketAddr) -> Self {
        Config {
            p2p: P2pConfig::default(p2p_addr),
            layer: LayerConfig::default(layer_addr),
            rpc: RpcConfig::default(rpc_addr),
        }
    }
}

pub fn new_channel() -> (Sender<Message>, Receiver<Message>) {
    channel::<Message>(MAX_MESSAGE_CAPACITY)
}

pub async fn start<G: 'static + Group>(
    out_send: Sender<Message>,
    group: G,
) -> Result<Sender<Message>> {
    let (send, recv) = new_channel();

    // TODO load config from configuare file
    let p2p_addr: SocketAddr = "0.0.0.0:8000".parse().unwrap();
    let layer_addr: SocketAddr = "0.0.0.0:8001".parse().unwrap();
    let rpc_addr: SocketAddr = "0.0.0.0:8002".parse().unwrap();

    let config = Config::default(p2p_addr, layer_addr, rpc_addr);

    task::spawn(start_main(out_send, recv, config, group));

    Ok(send)
}

pub async fn start_with_config<G: 'static + Group>(
    out_send: Sender<Message>,
    group: G,
    config: Config,
) -> Result<Sender<Message>> {
    let (send, recv) = new_channel();

    task::spawn(start_main(out_send, recv, config, group));

    Ok(send)
}

async fn start_main<G: 'static + Group>(
    out_send: Sender<Message>,
    self_recv: Receiver<Message>,
    config: Config,
    group: G,
) -> Result<()> {
    let (p2p_config, layer_config, rpc_config) = (config.p2p, config.layer, config.rpc);

    // start p2p
    // start layer_rpc
    // start inner json_rpc
    let (p2p_sender_result, layer_sender_result, rpc_sender_result) = join!(
        p2p_start(group, p2p_config, out_send.clone()),
        layer_start(layer_config, out_send.clone()),
        rpc_start(rpc_config, out_send)
    );
    let (p2p_sender, layer_sender, rpc_sender) =
        (p2p_sender_result?, layer_sender_result?, rpc_sender_result?);

    while let Some(message) = self_recv.recv().await {
        let sender = match message {
            Message::PeerJoin { .. } | Message::PeerLeave { .. } | Message::Event { .. } => {
                &p2p_sender
            }
            Message::Upper { .. } | Message::Lower { .. } | Message::Permission { .. } => {
                &layer_sender
            }
            Message::Rpc { .. } => &rpc_sender,
        };
        sender.send(message).await;
    }

    Ok(())
}
