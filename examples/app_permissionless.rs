use tdn::prelude::*;
//use tdn_permission::PermissionlessGroup;

#[tokio::main]
async fn main() {
    //let mut group = PermissionlessGroup::default();
    let (peer_addr, _send, mut out_recv) = start().await.unwrap();
    println!("Example: peer id: {}", peer_addr.short_show());

    while let Some(message) = out_recv.recv().await {
        match message {
            ReceiveMessage::Group(msg) => match msg {
                RecvType::Connect(_peer, _data) => {
                    //group.join(peer, data, send.clone()).await;
                }
                RecvType::Result(..) => {
                    //
                }
                RecvType::ResultConnect(_peer, _data) => {
                    //group.join(peer, data, send.clone()).await;
                }
                RecvType::Leave(_peer) => {
                    //group.leave(&peer);
                }
                RecvType::Event(peer_id, _data) => {
                    println!("receive group event from {}", peer_id.short_show());
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
