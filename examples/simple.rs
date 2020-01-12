use async_std::{sync::Arc, task};
use tdn::prelude::*;
use tdn::{new_channel, start};
use tdn_permission::PermissionlessGroup;

struct State(u32);

fn main() {
    task::block_on(async {
        let (out_send, out_recv) = new_channel();
        let mut group = PermissionlessGroup::default();
        let send = start(*group.id(), out_send).await.unwrap();

        let mut rpc_handler = RpcHandler::new(State(1));
        rpc_handler.add_method("echo", |params, state| {
            Box::pin(async move {
                assert_eq!(1, state.0);
                Ok(params)
            })
        });

        rpc_handler.add_method("say_hello", |_params, state| {
            Box::pin(async move {
                assert_eq!(1, state.0);
                Ok(RpcParam::String("Hello".to_owned()))
            })
        });

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
                    send.send(Message::Rpc(uid, rpc_handler.handle(params).await))
                        .await;
                }
                _ => {
                    println!("recv: {:?}", message);
                }
            }
        }
    });
}
