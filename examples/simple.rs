use std::sync::Arc;
use tdn::prelude::*;
use tdn::types::rpc::{json, RpcHandler};

struct State(u32);

fn main() {
    smol::block_on(async {
        let (peer_addr, send, out_recv) = start().await.unwrap();
        println!("Example: peer id: {}", peer_addr.short_show());

        let mut rpc_handler = RpcHandler::new(State(1));
        rpc_handler.add_method("echo", |params, state: Arc<State>| async move {
            assert_eq!(1, state.0);
            Ok(HandleResult::rpc(json!(params)))
        });

        rpc_handler.add_method("say_hello", |_params, state: Arc<State>| async move {
            assert_eq!(1, state.0);
            Ok(HandleResult::rpc(json!("hello")))
        });

        while let Ok(message) = out_recv.recv().await {
            match message {
                ReceiveMessage::Group(msg) => match msg {
                    GroupReceiveMessage::StableConnect(peer, _data) => {
                        println!("receive group peer {} join", peer.short_show());
                    }
                    GroupReceiveMessage::StableResult(..) => {
                        //
                    }
                    GroupReceiveMessage::StableLeave(peer) => {
                        println!("receive group peer {} leave", peer.short_show());
                    }
                    GroupReceiveMessage::Event(peer, _data) => {
                        println!("receive group event from {}", peer.short_show());
                    }
                    _ => {}
                },
                ReceiveMessage::Layer(gid, msg) => match msg {
                    LayerReceiveMessage::Connect(peer, _data) => {
                        println!(
                            "Layer Join: {}, Addr: {}.",
                            gid.short_show(),
                            peer.short_show()
                        );
                    }
                    LayerReceiveMessage::Result(..) => {
                        //
                    }
                    _ => {}
                },
                ReceiveMessage::Rpc(uid, params, is_ws) => {
                    if let Ok(HandleResult {
                        mut rpcs,
                        groups: _,
                        layers: _,
                    }) = rpc_handler.handle(params).await
                    {
                        loop {
                            if rpcs.len() != 0 {
                                let msg = rpcs.remove(0);
                                send.send(SendMessage::Rpc(uid, msg, is_ws))
                                    .await
                                    .expect("TDN channel closed");
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
        }
    });
}
