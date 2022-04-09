use std::sync::Arc;
use tdn::prelude::*;
use tdn::types::rpc::{json, RpcError, RpcHandler, RpcParam};

struct State(u32);

#[tokio::main]
async fn main() {
    let (peer_addr, send, mut out_recv) = start().await.unwrap();
    println!("Example: peer id: {:?}", peer_addr);

    let mut rpc_handler = RpcHandler::new(State(1));
    rpc_handler.add_method(
        "echo",
        |params: Vec<RpcParam>, state: Arc<State>| async move {
            let _value = params[0].as_str().ok_or(RpcError::ParseError)?;
            assert_eq!(1, state.0);
            Ok(HandleResult::rpc(json!(params)))
        },
    );

    rpc_handler.add_method("say_hello", |_params, state: Arc<State>| async move {
        assert_eq!(1, state.0);
        Ok(HandleResult::rpc(json!("hello")))
    });

    while let Some(message) = out_recv.recv().await {
        match message {
            ReceiveMessage::Own(msg) => match msg {
                RecvType::Connect(peer, _data) => {
                    println!("receive own peer {} join", peer.id.short_show());
                }
                RecvType::Leave(peer) => {
                    println!("receive own peer {} leave", peer.id.short_show());
                }
                RecvType::Event(peer_id, _data) => {
                    println!("receive own event from {}", peer_id.short_show());
                }
                _ => {
                    println!("nerver here!")
                }
            },
            ReceiveMessage::Group(msg) => match msg {
                RecvType::Connect(peer, _data) => {
                    println!("receive group peer {} join", peer.id.short_show());
                }
                RecvType::Result(..) => {
                    //
                }
                RecvType::Leave(peer) => {
                    println!("receive group peer {} leave", peer.id.short_show());
                }
                RecvType::Event(peer_id, _data) => {
                    println!("receive group event from {}", peer_id.short_show());
                }
                _ => {}
            },
            ReceiveMessage::Layer(fgid, _tgid, msg) => match msg {
                RecvType::Connect(peer, _data) => {
                    println!("Layer Join: {}, Addr: {}.", fgid, peer.id.short_show());
                }
                RecvType::Result(..) => {
                    //
                }
                _ => {}
            },
            ReceiveMessage::Rpc(uid, params, is_ws) => {
                if let Ok(HandleResult {
                    mut rpcs,
                    owns: _,
                    groups: _,
                    layers: _,
                    networks: _,
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
            ReceiveMessage::NetworkLost => {
                println!("No network connections");
            }
        }
    }
}
