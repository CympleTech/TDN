use std::collections::HashMap;
use std::marker::Send;

use crate::actor::prelude::*;
use crate::p2p::P2PActor;
use crate::primitives::functions::{try_resend_times, DEFAULT_TIMES};
use crate::primitives::types::GroupID;
use crate::rpc::RPCActor;
use crate::traits::actor::{BridgeActor, P2PBridgeActor, RPCBridgeActor};
use crate::traits::message::bridge_message::*;
use crate::traits::message::p2p_message::*;
use crate::traits::message::rpc_message::*;

#[derive(Clone)]
struct MultipleRecipient {
    upper_group: GroupID,
    lower_groups: Vec<GroupID>,

    recipient_event: Recipient<EventMessage>,
    recipient_peer_join: Recipient<PeerJoinMessage>,
    recipient_peer_join_result: Recipient<PeerJoinResultMessage>,
    recipient_peer_leave: Recipient<PeerLeaveMessage>,

    recipient_local: Recipient<LocalMessage>,
    recipient_upper: Recipient<UpperMessage>,
    recipient_lower: Recipient<LowerMessage>,

    recipient_local_response: Recipient<LocalResponseMessage>,
    recipient_upper_response: Recipient<UpperResponseMessage>,
    recipient_lower_response: Recipient<LowerResponseMessage>,
    recipient_level_permission: Recipient<LevelPermissionMessage>,
    recipient_level_permission_response: Recipient<LevelPermissionResponseMessage>,
}

#[derive(Clone)]
pub struct NetworkBridgeActor {
    p2p_addr: Addr<P2PActor<Self>>,
    rpc_addr: Addr<RPCActor<Self>>,
    bridges: HashMap<GroupID, MultipleRecipient>,
}

impl NetworkBridgeActor {
    pub fn load(p2p_addr: Addr<P2PActor<Self>>, rpc_addr: Addr<RPCActor<Self>>) -> Self {
        let bridges = HashMap::new();

        Self {
            p2p_addr,
            rpc_addr,
            bridges,
        }
    }

    /// try send received event to p2p actor
    fn send_p2p<M: 'static>(&self, message: M)
    where
        P2PActor<Self>: Handler<M>,
        M: Message + Send + Clone,
        <M as Message>::Result: Send,
        <P2PActor<Self> as Actor>::Context: ToEnvelope<P2PActor<Self>, M>,
    {
        let _ = try_resend_times(self.p2p_addr.clone(), message, DEFAULT_TIMES)
            .map_err(|_| println!("Send Message to udp fail"));
    }

    /// try send received event to rpc actor
    fn send_rpc<M: 'static>(&self, message: M)
    where
        RPCActor<Self>: Handler<M>,
        M: Message + Send + Clone,
        <M as Message>::Result: Send,
        <RPCActor<Self> as Actor>::Context: ToEnvelope<RPCActor<Self>, M>,
    {
        let _ = try_resend_times(self.rpc_addr.clone(), message, DEFAULT_TIMES)
            .map_err(|_| println!("Send Message to udp fail"));
    }
}

/// impl Actor for NetworkBridgeActor
impl Actor for NetworkBridgeActor {
    type Context = Context<Self>;

    /// when start register to p2p and rpc actor
    fn started(&mut self, ctx: &mut Self::Context) {
        self.send_p2p(P2PBridgeAddrMessage(ctx.address()));
        self.send_rpc(RPCBridgeAddrMessage(ctx.address()));
    }
}

/// impl BridgeActor for NetworkBridgeActor
impl BridgeActor for NetworkBridgeActor {}

/// receive local rpc request from bridge actor, and send to rpc
impl<B: BridgeActor> Handler<RegisterBridgeMessage<B>> for NetworkBridgeActor {
    type Result = bool;

    fn handle(&mut self, msg: RegisterBridgeMessage<B>, _ctx: &mut Self::Context) -> Self::Result {
        let (group_id, upper_group, addr) = (msg.0, msg.1, msg.2);
        let group = MultipleRecipient {
            upper_group: upper_group,
            lower_groups: Vec::new(),

            recipient_event: addr.clone().recipient::<EventMessage>(),
            recipient_peer_join: addr.clone().recipient::<PeerJoinMessage>(),
            recipient_peer_join_result: addr.clone().recipient::<PeerJoinResultMessage>(),
            recipient_peer_leave: addr.clone().recipient::<PeerLeaveMessage>(),

            recipient_local: addr.clone().recipient::<LocalMessage>(),
            recipient_upper: addr.clone().recipient::<UpperMessage>(),
            recipient_lower: addr.clone().recipient::<LowerMessage>(),

            recipient_local_response: addr.clone().recipient::<LocalResponseMessage>(),
            recipient_upper_response: addr.clone().recipient::<UpperResponseMessage>(),
            recipient_lower_response: addr.clone().recipient::<LowerResponseMessage>(),
            recipient_level_permission: addr.clone().recipient::<LevelPermissionMessage>(),
            recipient_level_permission_response: addr.recipient::<LevelPermissionResponseMessage>(),
        };

        if self.bridges.contains_key(&group_id) {
            false
        } else {
            self.bridges.insert(group_id, group);
            true
        }
    }
}

/// receive local rpc request from bridge actor, and send to rpc
impl Handler<LocalMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: LocalMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_rpc(ReceiveLocalMessage(msg.0, msg.1, msg.2, msg.3));
    }
}

/// receive send to upper rpc request from bridge actor, and send to rpc
impl Handler<UpperMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: UpperMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_rpc(ReceiveUpperMessage(msg.0, msg.1, msg.2));
    }
}

/// receive send to lower rpc request from bridge actor, and send to rpc
impl Handler<LowerMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: LowerMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_rpc(ReceiveLowerMessage(msg.0, msg.1, msg.2));
    }
}

/// receive send to lower rpc request from bridge actor, and send to rpc
impl Handler<LevelPermissionMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: LevelPermissionMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_rpc(ReceiveLevelPermissionMessage(msg.0, msg.1, msg.2, msg.3));
    }
}

impl Handler<LocalResponseMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: LocalResponseMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_rpc(ReceiveLocalResponseMessage(msg.0, msg.1, msg.2));
    }
}

impl Handler<UpperResponseMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: UpperResponseMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_rpc(ReceiveUpperResponseMessage(msg.0, msg.1, msg.2));
    }
}

impl Handler<LowerResponseMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: LowerResponseMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_rpc(ReceiveLowerResponseMessage(msg.0, msg.1, msg.2));
    }
}

impl Handler<LevelPermissionResponseMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: LevelPermissionResponseMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.send_rpc(ReceiveLevelPermissionResponseMessage(msg.0, msg.1, msg.2));
    }
}

/// receive event message from bridge actor, and send to p2p
impl Handler<EventMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: EventMessage, _ctx: &mut Self::Context) {
        self.send_p2p(ReceiveEventMessage(msg.0, msg.1, msg.2));
    }
}

/// receive peer join message from bridge actor, and send to p2p
impl Handler<PeerJoinMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: PeerJoinMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_p2p(ReceivePeerJoinMessage(msg.0, msg.1, msg.2, msg.3));
    }
}

/// receive peer join result from bridge actor, and send to p2p
impl Handler<PeerJoinResultMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: PeerJoinResultMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_p2p(ReceivePeerJoinResultMessage(msg.0, msg.1, msg.2, msg.3));
    }
}

/// receive peer leave message from bridge actor, and send to p2p
impl Handler<PeerLeaveMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: PeerLeaveMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.send_p2p(ReceivePeerLeaveMessage(msg.0, msg.1, msg.2));
    }
}

/// impl RPCBridgeActor for NetworkBridgeActor {}
impl P2PBridgeActor for NetworkBridgeActor {}

/// receive event message from p2p actor, and send to bridge
impl Handler<ReceiveEventMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: ReceiveEventMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.bridges.get(&msg.0).and_then(|group| {
            Some(
                group
                    .recipient_event
                    .do_send(EventMessage(msg.0, msg.1, msg.2)),
            )
        });
    }
}

/// receive peer join message from p2p actor, and send to bridge
impl Handler<ReceivePeerJoinMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: ReceivePeerJoinMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.bridges.get(&msg.0).and_then(|group| {
            Some(
                group
                    .recipient_peer_join
                    .do_send(PeerJoinMessage(msg.0, msg.1, msg.2, msg.3)),
            )
        });
    }
}

/// receive peer join result from bridge actor, and send to p2p
impl Handler<ReceivePeerJoinResultMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: ReceivePeerJoinResultMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.bridges.get(&msg.0).and_then(|group| {
            Some(
                group
                    .recipient_peer_join_result
                    .do_send(PeerJoinResultMessage(msg.0, msg.1, msg.2, msg.3)),
            )
        });
    }
}

/// receive peer leave message from p2p actor, and send to bridge
impl Handler<ReceivePeerLeaveMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: ReceivePeerLeaveMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.bridges.get(&msg.0).and_then(|group| {
            Some(
                group
                    .recipient_peer_leave
                    .do_send(PeerLeaveMessage(msg.0, msg.1, msg.2)),
            )
        });
    }
}

/// impl RPCBridgeActor for NetworkBridgeActor
impl RPCBridgeActor for NetworkBridgeActor {}

/// receive local rpc request from bridge actor, and send to rpc
impl Handler<ReceiveLocalMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: ReceiveLocalMessage, _ctx: &mut Self::Context) -> Self::Result {
        if self.bridges.contains_key(&msg.0) {
            let _ = self
                .bridges
                .get(&msg.0)
                .unwrap()
                .recipient_local
                .do_send(LocalMessage(msg.0, msg.1, msg.2, msg.3));
        } else {
            self.send_rpc(ReceiveLevelPermissionResponseMessage(msg.0, msg.1, false));
        }
    }
}

/// receive send to upper rpc request from bridge actor, and send to rpc
impl Handler<ReceiveUpperMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: ReceiveUpperMessage, _ctx: &mut Self::Context) -> Self::Result {
        if self.bridges.contains_key(&msg.0) {
            let _ = self
                .bridges
                .get(&msg.0)
                .unwrap()
                .recipient_upper
                .do_send(UpperMessage(msg.0, msg.1, msg.2));
        } else {
            self.send_rpc(ReceiveLevelPermissionResponseMessage(msg.0, msg.1, false));
        }
    }
}

/// receive send to lower rpc request from bridge actor, and send to rpc
impl Handler<ReceiveLowerMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(&mut self, msg: ReceiveLowerMessage, _ctx: &mut Self::Context) -> Self::Result {
        if self.bridges.contains_key(&msg.0) {
            let _ = self
                .bridges
                .get(&msg.0)
                .unwrap()
                .recipient_lower
                .do_send(LowerMessage(msg.0, msg.1, msg.2));
        } else {
            self.send_rpc(ReceiveLevelPermissionResponseMessage(msg.0, msg.1, false));
        }
    }
}

impl Handler<ReceiveLevelPermissionMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: ReceiveLevelPermissionMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        if self.bridges.contains_key(&msg.0) {
            let _ = self
                .bridges
                .get(&msg.0)
                .unwrap()
                .recipient_level_permission
                .do_send(LevelPermissionMessage(msg.0, msg.1, msg.2, msg.3));
        } else {
            self.send_rpc(ReceiveLevelPermissionResponseMessage(msg.0, msg.1, false));
        }
    }
}

impl Handler<ReceiveLocalResponseMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: ReceiveLocalResponseMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.bridges.get(&msg.0).and_then(|group| {
            Some(
                group
                    .recipient_local_response
                    .do_send(LocalResponseMessage(msg.0, msg.1, msg.2)),
            )
        });
    }
}

impl Handler<ReceiveUpperResponseMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: ReceiveUpperResponseMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.bridges.get(&msg.0).and_then(|group| {
            Some(
                group
                    .recipient_upper_response
                    .do_send(UpperResponseMessage(msg.0, msg.1, msg.2)),
            )
        });
    }
}

impl Handler<ReceiveLowerResponseMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: ReceiveLowerResponseMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.bridges.get(&msg.0).and_then(|group| {
            Some(
                group
                    .recipient_lower_response
                    .do_send(LowerResponseMessage(msg.0, msg.1, msg.2)),
            )
        });
    }
}

impl Handler<ReceiveLevelPermissionResponseMessage> for NetworkBridgeActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: ReceiveLevelPermissionResponseMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.bridges.get(&msg.0).and_then(|group| {
            Some(
                group
                    .recipient_level_permission_response
                    .do_send(LevelPermissionResponseMessage(msg.0, msg.1, msg.2)),
            )
        });
    }
}
