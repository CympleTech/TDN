#![recursion_limit = "1024"]
#![feature(associated_type_defaults)]

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
    PeerJoin(PeerAddr, SocketAddr, Vec<u8>),
    PeerJoinResult(PeerAddr, bool, Vec<u8>),
    PeerLeave(PeerAddr),
    Event(PeerAddr, Vec<u8>),
    Upper(GroupId, Vec<u8>),
    Lower(GroupId, Vec<u8>),
    Permission(GroupId, PeerAddr, SocketAddr),
    PermissionResult(GroupId, PeerAddr, bool),
    Rpc(Vec<u8>),
}

pub fn new_channel() -> (Sender<Message>, Receiver<Message>) {
    channel::<Message>(MAX_MESSAGE_CAPACITY)
}

// pub async fn multi_start(
//     groups: Vec<(GroupId, Sender<Message<impl Group>>)>,
// ) -> Result<Vec<Sender<Box<dyn Group>>>> {
//     for (gid, out_send) in groups {
//         let (send, recv) = new_channel();

//         let config = Config::load();

//         task::spawn(start_main(gid, out_send, recv, config));

//         Ok(send)
//     }
// }

pub async fn start(gid: GroupId, out_send: Sender<Message>) -> Result<Sender<Message>> {
    let (send, recv) = new_channel();

    let config = Config::load();

    task::spawn(start_main(gid, out_send, recv, config));

    Ok(send)
}

pub async fn start_with_config(
    gid: GroupId,
    out_send: Sender<Message>,
    config: Config,
) -> Result<Sender<Message>> {
    let (send, recv) = new_channel();

    task::spawn(start_main(gid, out_send, recv, config));

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
        layer_start(layer_config, out_send.clone()),
        rpc_start(rpc_config, out_send)
    );
    let (p2p_sender, layer_sender, rpc_sender) =
        (p2p_sender_result?, layer_sender_result?, rpc_sender_result?);

    while let Some(message) = self_recv.recv().await {
        let sender = match message {
            Message::PeerJoin { .. }
            | Message::PeerJoinResult { .. }
            | Message::PeerLeave { .. }
            | Message::Event { .. } => &p2p_sender,
            Message::Upper { .. }
            | Message::Lower { .. }
            | Message::Permission { .. }
            | Message::PermissionResult { .. } => &layer_sender,
            Message::Rpc { .. } => &rpc_sender,
        };
        sender.send(message).await;
    }

    Ok(())
}
