use async_std::{
    io::Result,
    sync::{channel, Receiver, Sender},
    task,
};
use chamomile::prelude::{start as p2p_start, PeerId, ReceiveMessage, SendMessage};
use futures::{select, FutureExt};

pub use chamomile::prelude::Config as P2pConfig;

use crate::message::{GroupMessage, GroupReceiveMessage, GroupSendMessage};
use crate::primitive::MAX_MESSAGE_CAPACITY;

/// new a channel, send message to p2p Message. default capacity is 1024.
fn new_send_channel() -> (Sender<GroupSendMessage>, Receiver<GroupSendMessage>) {
    channel(MAX_MESSAGE_CAPACITY)
}

pub(crate) async fn start<M: 'static + GroupMessage>(
    config: P2pConfig,
    out_send: Sender<M>,
) -> Result<(PeerId, Sender<GroupSendMessage>)> {
    let (self_send, self_recv) = new_send_channel();

    println!("DEBUG: P2P listening: {}", config.addr);

    // start chamomile
    let (peer_id, p2p_send, p2p_recv) = p2p_start(config).await?;
    println!("p2p service started");

    task::spawn(run_listen(out_send, p2p_send, p2p_recv, self_recv));
    println!("p2p channel service started");

    Ok((peer_id, self_send))
}

async fn run_listen<M: GroupMessage>(
    out_send: Sender<M>,
    p2p_send: Sender<SendMessage>,
    p2p_recv: Receiver<ReceiveMessage>,
    self_recv: Receiver<GroupSendMessage>,
) -> Result<()> {
    loop {
        select! {
            msg = p2p_recv.recv().fuse() => match msg {
                Some(msg) => {
                    match msg {
                        ReceiveMessage::PeerJoin(peer_addr, addr, data) => {
                            out_send.send(M::new_group(
                                GroupReceiveMessage::PeerJoin(peer_addr, addr, data)
                            )).await;
                        },
                        ReceiveMessage::PeerJoinResult(peer_addr, is_ok, data) => {
                            out_send.send(M::new_group(
                                GroupReceiveMessage::PeerJoinResult(peer_addr, is_ok, data)
                            )).await;
                        },
                        ReceiveMessage::PeerLeave(peer_addr) => {
                            out_send.send(M::new_group(
                                GroupReceiveMessage::PeerLeave(peer_addr)
                            )).await;
                        },
                        ReceiveMessage::Data(peer_addr, data) => {
                            println!("DEBUG: P2P Event Length: {}", data.len());
                            out_send.send(M::new_group(
                                GroupReceiveMessage::Event(peer_addr, data)
                            )).await;
                        }
                    }
                },
                None => break,
            },
            msg = self_recv.recv().fuse() => match msg {
                Some(msg) => {
                    match msg {
                        GroupSendMessage::PeerJoin(peer_addr, addr, data) => {
                            p2p_send.send(SendMessage::PeerJoin(peer_addr, addr, data)).await;
                        },
                        GroupSendMessage::PeerLeave(peer_addr) => {
                            p2p_send.send(SendMessage::PeerLeave(peer_addr)).await;
                        },
                        GroupSendMessage::PeerJoinResult(peer_addr, is_ok, is_force, result) => {
                            p2p_send.send(SendMessage::PeerJoinResult(
                                peer_addr, is_ok, is_force, result)).await;
                        },
                        GroupSendMessage::Connect(addr, data) => {
                            p2p_send.send(SendMessage::Connect(addr, data)).await;
                        },
                        GroupSendMessage::DisConnect(addr) => {
                            p2p_send.send(SendMessage::DisConnect(addr)).await;
                        },
                        GroupSendMessage::Event(peer_addr, data) => {
                            println!("DEBUG: Outside Event Length: {}", data.len());
                            p2p_send.send(SendMessage::Data(peer_addr, data)).await;
                        },
                        GroupSendMessage::Broadcast(broadcast, data) => {
                            p2p_send.send(SendMessage::Broadcast(broadcast, data)).await;
                        },
                    }
                },
                None => break,
            },
        }
    }

    drop(out_send);
    drop(p2p_send);
    drop(p2p_recv);
    drop(self_recv);

    Ok(())
}
