use std::net::SocketAddr;

use crate::actor::prelude::{Addr, Message};
use crate::primitives::types::{
    BlockByte, EventByte, EventID, GroupID, LevelPermissionByte, PeerAddr, PeerInfoByte, RPCParams,
};

use crate::traits::actor::BridgeActor;

/// event from p2p network self group.
/// Params is PeerAddr (p2p Node), Event Byte.
#[derive(Clone)]
pub struct EventMessage(pub GroupID, pub PeerAddr, pub EventByte);

impl Message for EventMessage {
    type Result = ();
}

/// peer join from p2p network.
/// Params is PeerAddr (p2p Node), Peer Join Info Byte.
#[derive(Clone)]
pub struct PeerJoinMessage(
    pub GroupID,
    pub PeerAddr,
    pub PeerInfoByte,
    pub Option<SocketAddr>,
);

impl Message for PeerJoinMessage {
    type Result = ();
}

/// peer join result when receive join request between p2p & bridge.
/// Params is PeerAddr (p2p Node), bool (join ok or not), help some peer addr.
#[derive(Clone)]
pub struct PeerJoinResultMessage(pub GroupID, pub PeerAddr, pub bool, pub Vec<PeerAddr>);

impl Message for PeerJoinResultMessage {
    type Result = ();
}

/// peer leave from p2p network.
/// Params is PeerAddr (p2p Node), bool if is true, lost by all peers,
/// if false, only first lost by self lost.
#[derive(Clone)]
pub struct PeerLeaveMessage(pub GroupID, pub PeerAddr, pub bool);

impl Message for PeerLeaveMessage {
    type Result = ();
}

/// rpc request from local outside, or send actor.
/// Params is SoocketAddr, RPCParams.
#[derive(Clone)]
pub struct LocalMessage(pub GroupID, pub usize, pub RPCParams, pub SocketAddr);

impl Message for LocalMessage {
    type Result = ();
}

/// rpc response from local outside or send to outsize.
/// Params is RPCParams.
#[derive(Clone)]
pub struct LocalResponseMessage(pub GroupID, pub usize, pub Option<RPCParams>);

impl Message for LocalResponseMessage {
    type Result = ();
}

/// rpc request from upper level group (send block for subscribed).
/// Params is rpc session_id, Block Byte.
#[derive(Clone)]
pub struct UpperMessage(pub GroupID, pub usize, pub BlockByte);

impl Message for UpperMessage {
    type Result = ();
}

/// rpc request from upper level group (send block for subscribed).
/// Params is EventID.
#[derive(Clone)]
pub struct UpperResponseMessage(pub GroupID, pub usize, pub Option<EventID>);

impl Message for UpperResponseMessage {
    type Result = ();
}

/// rpc request from lower level group (send block get more security).
/// Params is rpc session_id, Block Byte.
#[derive(Clone)]
pub struct LowerMessage(pub GroupID, pub usize, pub BlockByte);

impl Message for LowerMessage {
    type Result = ();
}

/// rpc request from lower level group (send block get more security).
/// Params is EventID.
#[derive(Clone)]
pub struct LowerResponseMessage(pub GroupID, pub usize, pub Option<EventID>);

impl Message for LowerResponseMessage {
    type Result = ();
}

/// rpc level permission request.
/// Params is LevelPermissionByte.
#[derive(Clone)]
pub struct LevelPermissionMessage(
    pub GroupID,
    pub usize,
    pub LevelPermissionByte,
    pub SocketAddr,
);

impl Message for LevelPermissionMessage {
    type Result = ();
}

/// rpc level permission response.
/// Params is LevelPermissionByte.
#[derive(Clone)]
pub struct LevelPermissionResponseMessage(pub GroupID, pub usize, pub bool);

impl Message for LevelPermissionResponseMessage {
    type Result = ();
}

#[derive(Clone)]
pub struct RegisterBridgeMessage<B: BridgeActor>(pub GroupID, pub GroupID, pub Addr<B>);

impl<B: BridgeActor> Message for RegisterBridgeMessage<B> {
    type Result = bool;
}
