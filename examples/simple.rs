use tdn::prelude::*;

struct State(u32);

fn main() {
    smol::block_on(async {
        let (peer_addr, send, out_recv) = start().await.unwrap();
        println!("Example: peer id: {}", peer_addr.short_show());

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

        while let Ok(message) = out_recv.recv().await {
            match message {
                ReceiveMessage::Group(msg) => match msg {
                    GroupReceiveMessage::PeerJoin(peer, addr, _data) => {
                        println!("receive group peer {} {:?} join", peer.short_show(), addr);
                    }
                    GroupReceiveMessage::PeerJoinResult(..) => {
                        //
                    }
                    GroupReceiveMessage::PeerLeave(peer) => {
                        println!("receive group peer {} leave", peer.short_show());
                    }
                    GroupReceiveMessage::Event(peer, _data) => {
                        println!("receive group event from {}", peer.short_show());
                    }
                    GroupReceiveMessage::Stream(..) => {
                        //
                    }
                },
                ReceiveMessage::Layer(msg) => match msg {
                    LayerReceiveMessage::LowerJoin(gid, remote_gid, uid, addr, join_data) => {
                        println!(
                            "Layer Join: {}, Addr: {}, join addr: {:?}",
                            gid.short_show(),
                            addr,
                            join_data
                        );
                        let _ = send
                            .send(SendMessage::Layer(LayerSendMessage::LowerJoinResult(
                                gid, remote_gid, uid, true,
                            )))
                            .await;
                    }
                    LayerReceiveMessage::LowerJoinResult(_gid, remote_gid, _uid, is_ok) => {
                        println!("Layer: {}, Join Result: {}", remote_gid.short_show(), is_ok);
                    }
                    _ => {}
                },
                ReceiveMessage::Rpc(uid, params, is_ws) => {
                    let _ = send
                        .send(SendMessage::Rpc(
                            uid,
                            rpc_handler.handle(params).await,
                            is_ws,
                        ))
                        .await;
                }
            }
        }
    });
}
