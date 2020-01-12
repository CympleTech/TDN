use async_std::task;
use tdn::{new_channel, start, Message};
use tdn_permission::PermissionlessGroup;

fn main() {
    task::block_on(async {
        let (out_send, out_recv) = new_channel();
        let mut group = PermissionlessGroup::default();
        let send = start(*group.id(), out_send).await.unwrap();

        while let Some(message) = out_recv.recv().await {
            match message {
                Message::PeerJoin(peer, addr, data) => {
                    group.join(peer, addr, data, send.clone()).await;
                }
                Message::PeerJoinResult(peer, is_ok, result) => {
                    group.join_result(peer, is_ok, result);
                }
                Message::PeerLeave(peer) => {
                    group.leave(&peer);
                }
                Message::Rpc(uid, params) => {
                    // echo
                    send.send(Message::Rpc(uid, params)).await;
                }
                _ => {
                    println!("recv: {:?}", message);
                }
            }
        }
    });
}
