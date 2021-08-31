use tdn::prelude::*;
use tdn_permission::PermissionlessGroup;

#[tokio::main]
async fn main() {
    let mut group = PermissionlessGroup::default();
    let (peer_addr, send, mut out_recv) = start().await.unwrap();
    println!("Example: peer id: {}", peer_addr.short_show());

    while let Some(message) = out_recv.recv().await {
        match message {
            ReceiveMessage::Group(msg) => match msg {
                RecvType::Connect(peer, data) => {
                    group.join(peer, data, send.clone()).await;
                }
                RecvType::Result(..) => {
                    //
                }
                RecvType::ResultConnect(peer, data) => {
                    group.join(peer, data, send.clone()).await;
                }
                RecvType::Leave(peer) => {
                    group.leave(&peer);
                }
                RecvType::Event(peer, _data) => {
                    println!("receive group event from {}", peer.short_show());
                }
                RecvType::Stream(..) => {
                    //
                }
                _ => {}
            },
            _ => {}
        }
    }
}
