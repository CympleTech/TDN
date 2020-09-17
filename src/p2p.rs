use chamomile::prelude::{start as p2p_start, ReceiveMessage, SendMessage};
use futures::{select, FutureExt};
use smol::channel::{self, Receiver, Sender};

pub use chamomile::prelude::Config as P2pConfig;

use tdn_types::{
    message::{GroupMessage, GroupReceiveMessage, GroupSendMessage},
    primitive::{PeerAddr, Result},
};

/// new a channel, send message to p2p Message. default capacity is 1024.
fn new_send_channel() -> (Sender<GroupSendMessage>, Receiver<GroupSendMessage>) {
    channel::unbounded()
}

pub(crate) async fn start<M: 'static + GroupMessage>(
    config: P2pConfig,
    out_send: Sender<M>,
) -> Result<(PeerAddr, Sender<GroupSendMessage>)> {
    let (self_send, self_recv) = new_send_channel();

    debug!("DEBUG: P2P listening: {}", config.addr);

    // start chamomile
    let (peer_id, p2p_send, p2p_recv) = p2p_start(config).await?;
    debug!("p2p service started");

    smol::spawn(run_listen(out_send, p2p_send, p2p_recv, self_recv)).detach();
    debug!("p2p channel service started");

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
                Ok(msg) => {
                    match msg {
                        ReceiveMessage::PeerJoin(peer_addr, addr, data) => {
                            out_send.send(M::new_group(
                                GroupReceiveMessage::PeerJoin(peer_addr, addr, data)
                            )).await.expect("P2P to Outside channel closed");
                        },
                        ReceiveMessage::PeerJoinResult(peer_addr, is_ok, data) => {
                            out_send.send(M::new_group(
                                GroupReceiveMessage::PeerJoinResult(peer_addr, is_ok, data)
                            )).await.expect("P2P to Outside channel closed");
                        },
                        ReceiveMessage::PeerLeave(peer_addr) => {
                            out_send.send(M::new_group(
                                GroupReceiveMessage::PeerLeave(peer_addr)
                            )).await.expect("P2P to Outside channel closed");
                        },
                        ReceiveMessage::Data(peer_addr, data) => {
                            debug!("DEBUG: P2P Event Length: {}", data.len());
                            out_send.send(M::new_group(
                                GroupReceiveMessage::Event(peer_addr, data)
                            )).await.expect("P2P to Outside channel closed");
                        }
                        ReceiveMessage::Stream(id, stream) => {
                            out_send.send(M::new_group(
                                GroupReceiveMessage::Stream(id, stream)
                            )).await.expect("P2P to Outside channel closed");
                        }
                    }
                },
                Err(_) => break,
            },
            msg = self_recv.recv().fuse() => match msg {
                Ok(msg) => {
                    match msg {
                        GroupSendMessage::PeerConnect(peer_addr, addr, data) => {
                            p2p_send.send(SendMessage::PeerConnect(peer_addr, addr, data))
                                .await.expect("P2P to chamomile channel closed");
                        },
                        GroupSendMessage::PeerDisconnect(peer_addr) => {
                            p2p_send.send(SendMessage::PeerDisconnect(peer_addr))
                                .await.expect("P2P to chamomile channel closed");
                        },
                        GroupSendMessage::PeerJoinResult(peer_addr, is_ok, is_force, result) => {
                            p2p_send.send(SendMessage::PeerJoinResult(
                                peer_addr, is_ok, is_force, result))
                                .await.expect("P2P to chamomile channel closed");
                        },
                        GroupSendMessage::Connect(addr, data) => {
                            p2p_send.send(SendMessage::Connect(addr, data))
                                .await.expect("P2P to chamomile channel closed");
                        },
                        GroupSendMessage::DisConnect(addr) => {
                            p2p_send.send(SendMessage::DisConnect(addr))
                                .await.expect("P2P to chamomile channel closed");
                        },
                        GroupSendMessage::Event(peer_addr, data) => {
                            debug!("DEBUG: Outside Event Length: {}", data.len());
                            p2p_send.send(SendMessage::Data(peer_addr, data))
                                .await.expect("P2P to chamomile channel closed");
                        },
                        GroupSendMessage::Broadcast(broadcast, data) => {
                            p2p_send.send(SendMessage::Broadcast(broadcast, data))
                                .await.expect("P2P to chamomile channel closed");
                        },
                        GroupSendMessage::Stream(id, stream) => {
                            p2p_send.send(SendMessage::Stream(id, stream))
                                .await.expect("P2P to chamomile channel closed");
                        }
                    }
                },
                Err(_) => break,
            },
        }
    }

    drop(out_send);
    drop(p2p_send);
    drop(p2p_recv);
    drop(self_recv);

    Ok(())
}
