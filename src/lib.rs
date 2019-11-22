use std::net::SocketAddr;
use async_std::sync::{channel, Sender, Receiver};
use async_std::io::Result;
use async_std::task;

mod p2p;
mod layer;
mod jsonrpc;

use p2p::P2pServer;
use layer::LayerServer;
use jsonrpc::JsonRpc;

pub const MAX_MESSAGE_CAPACITY: usize = 1024;

#[derive(Debug)]
pub struct PeerId;

#[derive(Debug)]
pub struct GroupId;

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
    Rpc(Vec<u8>)
}

pub fn new_channel() -> (Sender<Message>, Receiver<Message>) {
    channel::<Message>(MAX_MESSAGE_CAPACITY)
}

pub async fn start(out_send: Sender<Message>) -> Result<Sender<Message>> {
    let (send, recv) = new_channel();

    task::spawn(start_main(out_send, recv));

    Ok(send)
}

async fn start_main(out_send: Sender<Message>, self_recv: Receiver<Message>) -> Result<()> {
    // start p2p
    let p2p = P2pServer::new(out_send.clone());
    let p2p_sender = p2p.start().await?;

    // start layer_rpc
    let layer = LayerServer::new(out_send.clone());
    let layer_sender = layer.start().await?;

    // start inner json_rpc
    let rpc = JsonRpc::new(out_send.clone());
    let rpc_sender = rpc.start().await?;

    while let Some(message) = self_recv.recv().await {
        let sender = match message {
            Message::PeerJoin {..} | Message::PeerLeave {..} | Message::Event {..} => &p2p_sender,
            Message::Upper {..} | Message::Lower {..} | Message::Permission { .. } => &layer_sender,
            Message::Rpc { .. } => &rpc_sender
        };
        sender.send(message).await;
    }

    Ok(())
}
