use jsonrpc_parse::httpcodec::HTTPCodec;
use rand::{self, Rng};
use std::net::SocketAddr;
use tokio::codec::FramedRead;
use tokio::io::AsyncRead;
use tokio::net::TcpStream;

use crate::actor::prelude::*;
use crate::primitives::consts::{HIGH_WATERMARK, LOW_WATERMARK};
use crate::traits::actor::RPCBridgeActor;

use super::rpc::RPCActor;
use super::session::RPCSessionActor;

pub(crate) struct RPCListenActor<A: RPCBridgeActor> {
    pub rpc_addr: Addr<RPCActor<A>>,
}

impl<A: RPCBridgeActor> Actor for RPCListenActor<A> {
    type Context = Context<Self>;
}

pub(crate) struct RPCTcpConnectMessage(pub TcpStream, pub SocketAddr);

impl Message for RPCTcpConnectMessage {
    type Result = ();
}

impl<A: RPCBridgeActor> Handler<RPCTcpConnectMessage> for RPCListenActor<A> {
    type Result = ();

    fn handle(&mut self, msg: RPCTcpConnectMessage, _: &mut Context<Self>) {
        let rpc_addr = self.rpc_addr.clone();
        RPCSessionActor::create(move |ctx| {
            let id = rand::thread_rng().gen::<usize>();
            let (r, w) = msg.0.split();
            let read_frame = FramedRead::new(r, HTTPCodec::new());
            RPCSessionActor::add_stream(read_frame, ctx);
            let mut write_frame = FramedWrite::new(w, HTTPCodec::new(), ctx);
            write_frame.set_buffer_capacity(LOW_WATERMARK, HIGH_WATERMARK);
            RPCSessionActor::new(id, rpc_addr, write_frame, msg.1)
        });
    }
}

pub(crate) fn create_session<A: RPCBridgeActor>(
    id: usize,
    socket_addr: SocketAddr,
    rpc_addr: Addr<RPCActor<A>>,
) {
    Arbiter::spawn(
        TcpStream::connect(&socket_addr)
            .and_then(move |stream| {
                RPCSessionActor::create(move |ctx| {
                    let (r, w) = stream.split();
                    RPCSessionActor::add_stream(FramedRead::new(r, HTTPCodec::new()), ctx);

                    let mut write_frame = FramedWrite::new(w, HTTPCodec::new(), ctx);
                    write_frame.set_buffer_capacity(LOW_WATERMARK, HIGH_WATERMARK);
                    RPCSessionActor::new(id, rpc_addr, write_frame, socket_addr)
                });

                futures::future::ok(())
            })
            .map_err(|e| {
                println!("Cannot connect to peer : {}", e);
            }),
    );
}
