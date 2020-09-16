use tdn::prelude::*;
use tdn_permission::PermissionlessGroup;

fn main() {
    smol::block_on(async {
        let mut group = PermissionlessGroup::default();
        let (peer_addr, send, out_recv) = start().await.unwrap();
        println!("Example: peer id: {}", peer_addr.short_show());

        while let Ok(message) = out_recv.recv().await {
            match message {
                ReceiveMessage::Group(msg) => match msg {
                    GroupReceiveMessage::PeerJoin(peer, addr, data) => {
                        group.join(peer, addr, data, send.clone()).await;
                    }
                    GroupReceiveMessage::PeerJoinResult(..) => {
                        //
                    }
                    GroupReceiveMessage::PeerLeave(peer) => {
                        group.leave(&peer);
                    }
                    GroupReceiveMessage::Event(peer, _data) => {
                        println!("receive group event from {}", peer.short_show());
                    }
                    GroupReceiveMessage::Stream(..) => {
                        //
                    }
                },
                _ => {}
            }
        }
    });
}
