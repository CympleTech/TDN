use async_std::{
    io::Result,
    prelude::*,
    sync::{Receiver, Sender},
    task,
};
use futures::{select, FutureExt};
use serde::Deserialize;

pub use chamomile::Config as P2pConfig;
pub use chamomile::PeerId;
use chamomile::{new_channel as p2p_new_channel, start as p2p_start, Message as P2pMessage};

use crate::group::Group;
use crate::{new_channel, Message};

pub(crate) async fn start<G: 'static + Group>(
    group: G,
    config: P2pConfig,
    send: Sender<Message>,
) -> Result<Sender<Message>> {
    let (out_send, out_recv) = new_channel();
    let (p2p_send, p2p_recv) = p2p_new_channel();

    // start chamomile
    let p2p_send = p2p_start(p2p_send, config).await?;

    task::spawn(run_listen(group, send, p2p_send, p2p_recv, out_recv));

    Ok(out_send)
}

async fn run_listen<G: Group>(
    mut group: G,
    send: Sender<Message>,
    p2p_send: Sender<P2pMessage>,
    mut p2p_recv: Receiver<P2pMessage>,
    mut out_recv: Receiver<Message>,
) -> Result<()> {
    loop {
        select! {
            msg = p2p_recv.next().fuse() => match msg {
                Some(msg) => {
                    println!("recv from p2p: {:?}", msg);
                    match msg {
                        P2pMessage::PeerJoin(peer_addr, bytes) => {
                            let join_type_result = bincode::deserialize::<G::JoinType>(&bytes);
                            if join_type_result.is_ok()  {
                                let (is_ok, result) = group.join(peer_addr, join_type_result.unwrap());
                                let result_data = bincode::serialize(&result);
                                if is_ok && result_data.is_ok() {
                                    p2p_send.send(P2pMessage::PeerJoinResult(peer_addr, true, result_data.unwrap())).await;
                                    continue;
                                }
                            }

                            p2p_send.send(P2pMessage::PeerJoinResult(peer_addr, false, vec![])).await;
                        }
                        _ => {}
                    }
                    //send.send(msg).await;
                },
                None => break,
            },
            msg = out_recv.next().fuse() => match msg {
                Some(msg) => {
                    println!("recv from outside: {:?}", msg);
                    //p2p_send.send(msg).await;
                },
                None => break,
            },
        }
    }

    drop(group);
    drop(send);
    drop(p2p_send);
    drop(p2p_recv);
    drop(out_recv);

    Ok(())
}
