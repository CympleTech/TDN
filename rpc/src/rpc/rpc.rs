use rand::Rng;
use std::collections::HashMap;
use std::net::SocketAddr;

use crate::actor::prelude::*;
use crate::primitives::functions::{try_resend_times, DEFAULT_TIMES};
use crate::primitives::types::GroupID;
use crate::traits::actor::RPCBridgeActor;
use crate::traits::message::rpc_message::*;

use super::listen::create_session;
use super::request::Request;
use super::response::Response;
use super::session::{
    RPCSessionActor, RequestMessage, ResponseMessage, SessionCloseMessage, SessionOpenMessage,
};

/// RPC actor service.
/// it will handle every rpc request and response.
/// outside use donot need care how to send, only care send to response.
#[derive(Clone)]
pub struct RPCActor<A: RPCBridgeActor> {
    bridge: Option<Addr<A>>,
    sessions: HashMap<usize, Addr<RPCSessionActor<A>>>,
    waitings: HashMap<usize, Request>,
    upper_sockets: HashMap<GroupID, SocketAddr>, // TODO many choice
    lower_sockets: HashMap<GroupID, Vec<SocketAddr>>,
}

impl<A: RPCBridgeActor> RPCActor<A> {
    /// load new RPCActor object.
    pub fn load() -> Self {
        RPCActor {
            bridge: None,
            sessions: HashMap::new(),
            waitings: HashMap::new(),
            upper_sockets: HashMap::new(),
            lower_sockets: HashMap::new(),
        }
    }

    /// try send received request to network actor.
    fn send_bridge<M: 'static>(&self, message: M)
    where
        A: Handler<M>,
        M: Message + Send + Clone,
        <M as Message>::Result: Send,
        <A as Actor>::Context: ToEnvelope<A, M>,
    {
        if self.bridge.is_some() {
            let _ = try_resend_times(self.bridge.clone().unwrap(), message, DEFAULT_TIMES).map_err(
                |_| {
                    println!("Send request to network bridge fail");
                },
            );
        }
    }

    /// try response/request to session actor.
    fn send_session<M: 'static>(&self, index: usize, message: M)
    where
        RPCSessionActor<A>: Handler<M>,
        M: Message + Send + Clone,
        <M as Message>::Result: Send,
        <RPCSessionActor<A> as Actor>::Context: ToEnvelope<RPCSessionActor<A>, M>,
    {
        if self.sessions.get(&index).is_some() {
            let addr = self.sessions.get(&index).unwrap().clone();
            let _ = try_resend_times(addr, message, DEFAULT_TIMES).map_err(|_| {
                println!("Send request to session fail");
            });
        }
    }
}

/// impl Actor for RPCActor
impl<A: RPCBridgeActor> Actor for RPCActor<A> {
    type Context = Context<Self>;
}

impl<A: RPCBridgeActor> RPCBridgeActor for RPCActor<A> {}

/// when receive request from session, send to bridge.
impl<A: RPCBridgeActor> Handler<RequestMessage> for RPCActor<A> {
    type Result = ();

    fn handle(&mut self, msg: RequestMessage, _ctx: &mut Self::Context) -> Self::Result {
        let (index, request, socket_addr) = (msg.0, msg.1, msg.2);
        if self.sessions.get(&index).is_some() {
            match request {
                Request::Local(group, params) => {
                    self.send_bridge(ReceiveLocalMessage(group, index, params, socket_addr))
                }
                Request::Lower(group, block_bytes) => {
                    self.send_bridge(ReceiveLowerMessage(group, index, block_bytes))
                }
                Request::Upper(group, block_bytes) => {
                    self.send_bridge(ReceiveUpperMessage(group, index, block_bytes))
                }
                Request::Permission(group, permission_bytes) => self.send_bridge(
                    ReceiveLevelPermissionMessage(group, index, permission_bytes, socket_addr),
                ),
                _ => {}
            }
        }
    }
}

/// when receive response from session, send to bridge.
impl<A: RPCBridgeActor> Handler<ResponseMessage> for RPCActor<A> {
    type Result = ();

    fn handle(&mut self, msg: ResponseMessage, _ctx: &mut Self::Context) -> Self::Result {
        let (index, response) = (msg.0, msg.1);
        if self.sessions.get(&index).is_some() {
            match response {
                Response::Local(group, params) => {
                    self.send_bridge(ReceiveLocalResponseMessage(group, index, Some(params)))
                }
                Response::Lower(group, event_id) => {
                    self.send_bridge(ReceiveLowerResponseMessage(group, index, event_id))
                }
                Response::Upper(group, event_id) => {
                    self.send_bridge(ReceiveUpperResponseMessage(group, index, event_id))
                }
                Response::Permission(group, permission) => {
                    // TODO save permission result.
                    self.send_bridge(ReceiveLevelPermissionResponseMessage(
                        group, index, permission,
                    ))
                }
                _ => {}
            }
        }
    }
}

/// when session create, save it. and send waiting task to it.
impl<A: RPCBridgeActor> Handler<SessionOpenMessage<A>> for RPCActor<A> {
    type Result = ();

    fn handle(&mut self, msg: SessionOpenMessage<A>, _ctx: &mut Self::Context) -> Self::Result {
        let (index, addr) = (msg.0, msg.1);
        self.sessions.insert(index, addr);
        if self.waitings.contains_key(&index) {
            let request = self.waitings.remove(&index).unwrap();
            self.send_session(
                index,
                RequestMessage(index, request, "0.0.0.0:0".parse().unwrap()), // use default sock because dono use
            );
        }
    }
}

/// when session close, delete it.
impl<A: RPCBridgeActor> Handler<SessionCloseMessage> for RPCActor<A> {
    type Result = ();

    fn handle(&mut self, msg: SessionCloseMessage, _ctx: &mut Self::Context) -> Self::Result {
        let index = msg.0;
        self.sessions.remove(&index);
        self.waitings.remove(&index);
    }
}

impl<A: RPCBridgeActor> Handler<RPCBridgeAddrMessage<A>> for RPCActor<A> {
    type Result = ();

    fn handle(&mut self, msg: RPCBridgeAddrMessage<A>, _ctx: &mut Self::Context) -> Self::Result {
        self.bridge = Some(msg.0);
    }
}

impl<A: RPCBridgeActor> Handler<ReceiveLocalMessage> for RPCActor<A> {
    type Result = ();

    fn handle(&mut self, msg: ReceiveLocalMessage, ctx: &mut Self::Context) -> Self::Result {
        let (group, params, socket_addr) = (msg.0, msg.2, msg.3);
        let id = rand::thread_rng().gen::<usize>();
        let request = Request::Local(group, params);
        self.waitings.insert(id, request);
        let rpc_addr = ctx.address();

        create_session(id, socket_addr, rpc_addr);
    }
}

impl<A: RPCBridgeActor> Handler<ReceiveUpperMessage> for RPCActor<A> {
    type Result = ();

    fn handle(&mut self, msg: ReceiveUpperMessage, ctx: &mut Self::Context) -> Self::Result {
        let (group, block_bytes) = (msg.0, msg.2);
        if let Some(socket_addr) = self.upper_sockets.get(&group) {
            let id = rand::thread_rng().gen::<usize>();
            let request = Request::Upper(group, block_bytes);
            self.waitings.insert(id, request);
            let rpc_addr = ctx.address();

            create_session(id, socket_addr.clone(), rpc_addr);
        }
    }
}

impl<A: RPCBridgeActor> Handler<ReceiveLowerMessage> for RPCActor<A> {
    type Result = ();

    fn handle(&mut self, msg: ReceiveLowerMessage, ctx: &mut Self::Context) -> Self::Result {
        let (group, block_bytes) = (msg.0, msg.2);
        if let Some(socket_addrs) = self.lower_sockets.get(&group) {
            for socket_addr in socket_addrs.iter() {
                let id = rand::thread_rng().gen::<usize>();
                let request = Request::Lower(group.clone(), block_bytes.clone());
                self.waitings.insert(id, request);
                let rpc_addr = ctx.address();

                create_session(id, socket_addr.clone(), rpc_addr);
            }
        }
    }
}

impl<A: RPCBridgeActor> Handler<ReceiveLevelPermissionMessage> for RPCActor<A> {
    type Result = ();

    fn handle(
        &mut self,
        msg: ReceiveLevelPermissionMessage,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let (group, permission_bytes, socket_addr) = (msg.0, msg.2, msg.3);
        let id = rand::thread_rng().gen::<usize>();
        let request = Request::Permission(group, permission_bytes);
        self.waitings.insert(id, request);
        let rpc_addr = ctx.address();

        create_session(id, socket_addr, rpc_addr);
    }
}

impl<A: RPCBridgeActor> Handler<ReceiveLocalResponseMessage> for RPCActor<A> {
    type Result = ();

    fn handle(
        &mut self,
        msg: ReceiveLocalResponseMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let (group, index, result) = (msg.0, msg.1, msg.2);
        let response = if result.is_some() {
            Response::Local(group, result.unwrap())
        } else {
            Response::Invalid
        };

        self.send_session(index, ResponseMessage(0usize, response));
    }
}

impl<A: RPCBridgeActor> Handler<ReceiveLowerResponseMessage> for RPCActor<A> {
    type Result = ();

    fn handle(
        &mut self,
        msg: ReceiveLowerResponseMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let (group, index, result) = (msg.0, msg.1, msg.2);
        self.send_session(
            index,
            ResponseMessage(0usize, Response::Lower(group, result)),
        );
    }
}

impl<A: RPCBridgeActor> Handler<ReceiveUpperResponseMessage> for RPCActor<A> {
    type Result = ();

    fn handle(
        &mut self,
        msg: ReceiveUpperResponseMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let (group, index, result) = (msg.0, msg.1, msg.2);
        self.send_session(
            index,
            ResponseMessage(0usize, Response::Upper(group, result)),
        );
    }
}

impl<A: RPCBridgeActor> Handler<ReceiveLevelPermissionResponseMessage> for RPCActor<A> {
    type Result = ();

    fn handle(
        &mut self,
        msg: ReceiveLevelPermissionResponseMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let (group, index, result) = (msg.0, msg.1, msg.2);
        self.send_session(
            index,
            ResponseMessage(0usize, Response::Permission(group, result)),
        );
    }
}
