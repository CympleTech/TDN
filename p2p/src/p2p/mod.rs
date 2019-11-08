use futures::Stream;
use std::net::SocketAddr;
use tokio::codec::BytesCodec;
use tokio::net::{UdpFramed, UdpSocket};

use crate::actor::prelude::*;
use crate::crypto::keypair::PrivateKey;
use crate::traits::actor::P2PBridgeActor;

mod codec;
mod content;
mod dht;
mod p2p;
mod session;

pub use p2p::P2PActor;
pub use session::{CodecMessage, P2PSessionActor};

pub fn p2p_start<B: P2PBridgeActor>(
    p2p_socket: SocketAddr,
    psk: Option<PrivateKey>,
) -> Addr<P2PActor<B>> {
    // bind to udp
    let sock =
        UdpSocket::bind(&p2p_socket).expect(&format!("P2P Socket bind: {} fail!", p2p_socket));

    // start p2p session
    let (sink, stream) = UdpFramed::new(sock, BytesCodec::new()).split();
    let session_addr = P2PSessionActor::create(|ctx| {
        ctx.set_mailbox_capacity(100);
        ctx.add_stream(stream.map(|(data, sender)| CodecMessage(data, sender)));
        P2PSessionActor {
            sinks: vec![sink],
            p2p_addr: None,
            waitings: vec![],
            receivings: Default::default(),
        }
    });

    println!("DEBUG: P2P listen: {}", p2p_socket);
    // start p2p actor
    P2PActor::create(|ctx| {
        ctx.set_mailbox_capacity(100);
        P2PActor::load(session_addr, psk)
    })
}
