use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::actor::prelude::*;
use crate::traits::actor::RPCBridgeActor;

mod listen;
mod request;
mod response;
mod rpc;
mod session;

use listen::{RPCListenActor, RPCTcpConnectMessage};
pub use rpc::RPCActor;

pub fn rpc_start<A: RPCBridgeActor>(rpc_socket: SocketAddr) -> Addr<RPCActor<A>> {
    // start rpc actor
    let rpc_addr = RPCActor::create(|ctx: &mut Context<RPCActor<A>>| {
        ctx.set_mailbox_capacity(100);
        RPCActor::load()
    });

    // listen RPC TCP socket
    let listener =
        TcpListener::bind(&rpc_socket).expect(&format!("RPC Socket bind: {} fail!", rpc_socket));

    let new_rpc_addr = rpc_addr.clone();

    println!("DEBUG: RPC listen: {}", rpc_socket);
    // start rpc session actor
    RPCListenActor::create(|ctx| {
        ctx.set_mailbox_capacity(100);
        ctx.add_message_stream(listener.incoming().map_err(|_| ()).map(|st| {
            let addr = st.peer_addr().unwrap();
            RPCTcpConnectMessage(st, addr)
        }));
        RPCListenActor {
            rpc_addr: new_rpc_addr,
        }
    });

    rpc_addr
}
