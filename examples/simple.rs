use async_std::task;
use async_std::sync::channel;
use tdn::{new_channel, start, Group, PeerId, GroupId};

#[derive(Default)]
struct SimpleGroup {
    id: GroupId,
    peers: Vec<PeerId>,
}

impl Group for SimpleGroup {
    type JoinType = String;

    fn id(&self) -> &GroupId {
        &self.id
    }

    /// directly add a peer to group.
    fn add(&mut self, peer_id: &PeerId) {
        self.peers.push(peer_id.clone());
    }
}


fn main() {
    task::block_on(async {
        let (out_send, out_recv) = new_channel();
        let send = start(out_send, SimpleGroup::default()).await.unwrap();

        while let Some(message) = out_recv.recv().await {
            println!("recv: {:?}", message);
            send.send(message).await;
        }
    });
}
