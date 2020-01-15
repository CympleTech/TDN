use async_std::{
    io::Result,
    prelude::*,
    sync::{Receiver, Sender},
    task,
};
use futures::{select, FutureExt};

pub use chamomile::Config as P2pConfig;
use chamomile::{new_channel as p2p_new_channel, start as p2p_start, Message as P2pMessage};

use crate::primitive::GroupId;
use crate::{new_channel, GroupMessage, Message};

pub(crate) async fn start(
    gid: GroupId,
    config: P2pConfig,
    send: Sender<Message>,
) -> Result<Sender<Message>> {
    let (out_send, out_recv) = new_channel();
    let (p2p_send, p2p_recv) = p2p_new_channel();

    println!(
        "DEBUG: Group: {:?} is running, P2P listening: {}",
        gid.short_show(),
        config.addr
    );

    // start chamomile
    let p2p_send = p2p_start(p2p_send, config).await?;

    task::spawn(run_listen(gid, send, p2p_send, p2p_recv, out_recv));

    Ok(out_send)
}

async fn run_listen(
    _gid: GroupId,
    send: Sender<Message>,
    p2p_send: Sender<P2pMessage>,
    mut p2p_recv: Receiver<P2pMessage>,
    mut out_recv: Receiver<Message>,
) -> Result<()> {
    loop {
        select! {
            msg = p2p_recv.next().fuse() => match msg {
                Some(msg) => {
                    match msg {
                        P2pMessage::PeerJoin(peer_addr, addr, data) => {
                            send.send(Message::Group(GroupMessage::PeerJoin(peer_addr, addr, data))).await;
                        },
                        P2pMessage::PeerJoinResult(peer_addr, is_ok, result) => {
                            send.send(Message::Group(GroupMessage::PeerJoinResult(peer_addr, is_ok, result))).await;
                        },
                        P2pMessage::PeerLeave(peer_addr) => {
                            send.send(Message::Group(GroupMessage::PeerLeave(peer_addr))).await;
                        }
                        P2pMessage::Data(peer_addr, data) => {
                            println!("DEBUG: P2P Event Length: {}", data.len());
                            send.send(Message::Group(GroupMessage::Event(peer_addr, data))).await;
                        }
                        _ => {} // others not handle
                    }
                },
                None => break,
            },
            msg = out_recv.next().fuse() => match msg {
                Some(msg) => {
                    match msg {
                        Message::Group(message) => {
                            match message {
                                GroupMessage::PeerJoinResult(peer_addr, is_ok, result) => {
                                    p2p_send.send(P2pMessage::PeerJoinResult(peer_addr, is_ok, result)).await;
                                },
                                GroupMessage::PeerJoin(peer_addr, addr, data) => {
                                    p2p_send.send(P2pMessage::PeerJoin(peer_addr, addr, data)).await;
                                }
                                GroupMessage::PeerLeave(peer_addr) => {
                                    p2p_send.send(P2pMessage::PeerLeave(peer_addr)).await;
                                }
                                GroupMessage::Event(peer_addr, data) => {
                                    println!("DEBUG: Outside Event Length: {}", data.len());
                                    p2p_send.send(P2pMessage::Data(peer_addr, data)).await;
                                }
                            }
                        }
                        _ => {} // others not handle
                    }
                },
                None => break,
            },
        }
    }

    drop(send);
    drop(p2p_send);
    drop(p2p_recv);
    drop(out_recv);

    Ok(())
}
