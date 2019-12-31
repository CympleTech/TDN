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

pub mod error;
pub mod primitive;

use jsonrpc::start as rpc_start;
use layer::start as layer_start;
use p2p::start as p2p_start;
use primitive::MAX_MESSAGE_CAPACITY;

pub use config::Config;
pub use group::{Group, GroupId};
pub use p2p::PeerId as PeerAddr;

#[derive(Debug)]
pub struct PeerInfo;

#[derive(Debug)]
pub enum Message {
    PeerJoin(GroupId, PeerAddr, PeerInfo, Option<SocketAddr>),
    PeerLeave(GroupId, PeerAddr),
    Event(GroupId, PeerAddr, Vec<u8>),
    Upper(GroupId, PeerAddr, Vec<u8>),
    Lower(GroupId, PeerAddr, Vec<u8>),
    Permission(GroupId, PeerAddr, Option<SocketAddr>),
    Rpc(Vec<u8>),
}

pub fn new_channel() -> (Sender<Message>, Receiver<Message>) {
    channel::<Message>(MAX_MESSAGE_CAPACITY)
}

pub async fn start<G: 'static + Group>(
    out_send: Sender<Message>,
    group: G,
) -> Result<Sender<Message>> {
    let (send, recv) = new_channel();

    let config = Config::load();

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
    let (p2p_config, layer_config, rpc_config) = config.split();

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
