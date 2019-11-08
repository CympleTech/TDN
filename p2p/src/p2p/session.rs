use bytes::{BufMut, BytesMut};
use futures::stream::SplitSink;
use futures::Sink;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::codec::BytesCodec;
use tokio::net::UdpFramed;

use crate::actor::prelude::*;
use crate::primitives::functions::{try_resend_times, DEFAULT_TIMES};
use crate::traits::actor::P2PBridgeActor;

use super::codec::{P2PBody, P2PHead, HEAD_LENGTH};
use super::content::P2PContent;
use super::p2p::P2PActor;

/// message between session and p2p actor.
#[derive(Clone)]
pub struct P2PMessage(pub P2PHead, pub P2PContent, pub SocketAddr);

impl Message for P2PMessage {
    type Result = ();
}

/// message between session and UPD listen.
#[derive(Clone)]
pub struct CodecMessage(pub BytesMut, pub SocketAddr);

impl Message for CodecMessage {
    type Result = ();
}

/// p2p addr message, need register to p2p session
#[derive(Clone)]
pub(crate) struct P2PAddrMessage<A: P2PBridgeActor>(pub Addr<P2PActor<A>>);

impl<A: P2PBridgeActor> Message for P2PAddrMessage<A> {
    type Result = ();
}

pub struct P2PSessionActor<A: P2PBridgeActor> {
    pub sinks: Vec<SplitSink<UdpFramed<BytesCodec>>>,
    pub p2p_addr: Option<Addr<P2PActor<A>>>,
    pub waitings: Vec<(P2PHead, P2PBody, SocketAddr)>,
    pub receivings: HashMap<[u8; 8], Vec<u8>>,
}

impl<A: P2PBridgeActor> P2PSessionActor<A> {
    fn send_udp(
        &mut self,
        mut bytes: Vec<u8>,
        mut prev_sign: [u8; 8],
        self_sign: [u8; 8],
        socket: SocketAddr,
        ctx: &mut Context<Self>,
    ) {
        let mut send_bytes = vec![];
        let (mut now, next, next_sign) = if bytes.len() > 65400 {
            let now = bytes.drain(0..65400).as_slice().into();
            (now, bytes, rand::random())
        } else {
            (bytes, vec![], self_sign.clone())
        };

        send_bytes.extend_from_slice(&mut prev_sign);
        send_bytes.extend_from_slice(&mut self_sign.clone());
        send_bytes.extend_from_slice(&mut next_sign.clone());
        send_bytes.append(&mut now);

        let mut dst = BytesMut::new();
        dst.reserve(send_bytes.len());
        dst.put(send_bytes);
        self.sinks.pop().and_then(|sink| {
            let _ = sink
                .send((dst.into(), socket.clone()))
                .into_actor(self)
                .then(move |res, act, ctx| {
                    match res {
                        Ok(sink) => {
                            act.sinks.push(sink);
                            if !next.is_empty() {
                                act.send_udp(next, self_sign, next_sign, socket, ctx);
                            }
                        }
                        Err(_) => panic!("DEBUG: NETWORK HAVE ERROR"),
                    }

                    actor_ok(())
                })
                .wait(ctx);
            Some(())
        });
    }
}

impl<A: P2PBridgeActor> Actor for P2PSessionActor<A> {
    type Context = Context<Self>;
}

/// when receive P2PMessage, send it to that socket.
impl<A: P2PBridgeActor> Handler<P2PAddrMessage<A>> for P2PSessionActor<A> {
    type Result = ();

    fn handle(&mut self, msg: P2PAddrMessage<A>, _ctx: &mut Context<Self>) {
        self.p2p_addr = Some(msg.0);
    }
}

/// when receive from upd stream, send to p2p actor to handle.
impl<A: P2PBridgeActor> StreamHandler<CodecMessage, std::io::Error> for P2PSessionActor<A> {
    fn handle(&mut self, msg: CodecMessage, _ctx: &mut Context<Self>) {
        let (mut src, socket) = (msg.0, msg.1);
        if src.len() < 16 {
            return;
        }
        let (head_sign, new_data) = src.split_at_mut(24);

        let (prev, me_next) = head_sign.split_at_mut(8);
        let (me, next) = me_next.split_at_mut(8);

        let mut prev_sign = [0u8; 8];
        prev_sign.copy_from_slice(prev);
        let mut sign = [0u8; 8];
        sign.copy_from_slice(me);
        let mut next_sign = [0u8; 8];
        next_sign.copy_from_slice(next);

        let mut data = vec![];

        if let Some(mut prev_data) = self.receivings.remove(&prev_sign) {
            prev_sign = sign;
            data.append(&mut prev_data);
        }

        data.extend_from_slice(new_data);

        if let Some(mut next_data) = self.receivings.remove(&next_sign) {
            data.append(&mut next_data);
        }

        let head = {
            if data.len() < HEAD_LENGTH || prev_sign != sign {
                self.receivings.insert(sign, data);
                return;
            }
            P2PHead::decode(data.as_ref())
        };

        let size = head.len as usize;

        if data.len() >= size + HEAD_LENGTH {
            let (_, data) = data.split_at_mut(HEAD_LENGTH);
            let (buf, _) = data.split_at_mut(size);

            let content = bincode::deserialize(buf).unwrap_or(P2PContent::None);
            if self.p2p_addr.is_some() {
                let _ = try_resend_times(
                    self.p2p_addr.clone().unwrap(),
                    P2PMessage(head, content, socket),
                    DEFAULT_TIMES,
                )
                .map_err(|_| {
                    println!("Send Message to p2p fail");
                });
            }
        } else {
            self.receivings.insert(sign, data);
        }
    }
}

/// when receive P2PMessage, send it to that socket.
impl<A: P2PBridgeActor> Handler<P2PMessage> for P2PSessionActor<A> {
    type Result = ();

    fn handle(&mut self, msg: P2PMessage, ctx: &mut Context<Self>) {
        self.waitings.push((msg.0, P2PBody(msg.1), msg.2));
        if self.sinks.is_empty() {
            return;
        }

        while !self.waitings.is_empty() {
            let w = self.waitings.remove(0);
            if self.sinks.is_empty() {
                self.waitings.push(w);
                break;
            }
            let (mut head, body, socket) = (w.0, w.1, w.2);

            let mut body_bytes: Vec<u8> = bincode::serialize(&body).unwrap_or(vec![]);
            head.update_len(body_bytes.len() as u32);
            let mut head_bytes = head.encode().to_vec();
            let mut bytes = vec![];
            bytes.append(&mut head_bytes);
            bytes.append(&mut body_bytes);

            let sign: [u8; 8] = rand::random();

            self.send_udp(bytes, sign.clone(), sign, socket, ctx);
        }
    }
}
