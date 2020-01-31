use async_std::task;
use tdn::prelude::*;
use tdn::{new_channel, start};
use tdn_permission::PermissionlessGroup;

struct State(u32);

fn main() {
    task::block_on(async {
        let (out_send, out_recv) = new_channel();
        let mut group = PermissionlessGroup::default();
        let send = start(out_send).await.unwrap();

        let mut rpc_handler = RpcHandler::new(State(1));
        rpc_handler.add_method("echo", |params, state| {
            Box::pin(async move {
                assert_eq!(1, state.0);
                Ok(RpcParam::Array(params))
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
                Message::Group(msg) => match msg {
                    GroupMessage::PeerJoin(peer, addr, data) => {
                        group.join(peer, addr, data, send.clone()).await;
                    }
                    GroupMessage::PeerJoinResult(peer, is_ok, result) => {
                        group.join_result(peer, is_ok, result);
                    }
                    GroupMessage::PeerLeave(peer) => {
                        group.leave(&peer);
                    }
                    _ => {}
                },
                Message::Layer(msg) => match msg {
                    LayerMessage::LowerJoin(gid, uid, addr, join_data) => {
                        println!(
                            "Layer Join: {}, Addr: {}, join addr: {:?}",
                            gid.short_show(),
                            addr,
                            join_data
                        );
                        send.send(Message::Layer(LayerMessage::LowerJoinResult(
                            gid, uid, true,
                        )))
                        .await;
                    }
                    LayerMessage::LowerJoinResult(gid, _uid, is_ok) => {
                        println!("Layer: {}, Join Result: {}", gid.short_show(), is_ok);
                    }
                    _ => {}
                },
                Message::Rpc(uid, params, _is_ws) => {
                    send.send(Message::Rpc(uid, rpc_handler.handle(params).await, false))
                        .await;
                }
            }
        }
    });
}
