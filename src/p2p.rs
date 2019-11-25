use std::net::SocketAddr;
use async_std::sync::{Sender, Receiver, Arc, Mutex};
use async_std::io::Result;
use async_std::task;

use chamomile::{start, new_channel as p2p_new_channel, Message as P2pMessage};
pub use chamomile::Config as P2pConfig;
pub use chamomile::PeerId;

use crate::group::Group;
use crate::{Message, new_channel};

pub(crate) struct P2pServer<G: 'static + Group> {
    group: G,
    config: P2pConfig,
}

impl<G: 'static + Group> P2pServer<G> {
    pub fn new(group: G, config: P2pConfig) -> Self {
        Self { group, config }
    }

    pub async fn start(server: P2pServer<G>, out_send: Sender<Message>) -> Result<Sender<Message>> {
        let (send, recv) = new_channel();
        let (self_send, self_recv) = p2p_new_channel();

        // start chamomile
        let p2p_send = start(self_send, server.config.clone()).await?;

        let m1 = Arc::new(Mutex::new(server));
        let m2 = m1.clone();

        // start listen self recv
        task::spawn(run_listen_p2p(m1, out_send.clone(), p2p_send.clone(), self_recv));

        // start listen outside recv
        task::spawn(run_listen_outside(m2, out_send, p2p_send, recv));

        Ok(send)
    }
}


async fn run_listen_p2p<G: Group>(
    server: Arc<Mutex<P2pServer<G>>>,
    out_send: Sender<Message>,
    send: Sender<P2pMessage>,
    recv: Receiver<P2pMessage>
) -> Result<()> {
    while let Some(message) = recv.recv().await {
        println!("recv from p2p: {:?}", message);
        send.send(message).await;
    }
    Ok(())
}

async fn run_listen_outside<G: Group>(
    server: Arc<Mutex<P2pServer<G>>>,
    out_send: Sender<Message>,
    send: Sender<P2pMessage>,
    recv: Receiver<Message>
) -> Result<()> {
    while let Some(message) = recv.recv().await {
        println!("recv from outisade: {:?}", message);
        //send.send(message).await;
    }
    Ok(())
}
