use std::net::SocketAddr;
use async_std::sync::{Sender, Receiver};
use async_std::io::Result;
use async_std::task;

use chamomile::{start, new_channel as p2p_new_channel, Message as P2pMessage};
pub use chamomile::Config as P2pConfig;
pub use chamomile::PeerId;

use crate::group::Group;
use crate::{Message, new_channel};

pub(crate) struct P2pServer<G: Group> {
    out_send: Sender<Message>,
    group: G,
    config: P2pConfig,
}

impl<G: Group> P2pServer<G> {
    pub fn new(out_send: Sender<Message>, group: G, config: P2pConfig) -> Self {
        Self { out_send, group, config }
    }

    pub async fn start(&mut self) -> Result<Sender<Message>> {
        let (send, recv) = new_channel();
        let (self_send, self_recv) = p2p_new_channel();

        // start chamomile
        let p2p_send = start(self_send, self.config.clone()).await?;

        // start listen self recv
        task::spawn(run_listen_p2p(p2p_send.clone(), self_recv));

        // start listen outside recv
        task::spawn(run_listen_outside(p2p_send, recv));

        Ok(send)
    }
}


async fn run_listen_p2p(
    send: Sender<P2pMessage>,
    recv: Receiver<P2pMessage>
) -> Result<()> {
    while let Some(message) = recv.recv().await {
        println!("recv from p2p: {:?}", message);
        send.send(message).await;
    }
    Ok(())
}

async fn run_listen_outside(
    send: Sender<P2pMessage>,
    recv: Receiver<Message>
) -> Result<()> {
    while let Some(message) = recv.recv().await {
        println!("recv from outisade: {:?}", message);
        //send.send(message).await;
    }
    Ok(())
}
