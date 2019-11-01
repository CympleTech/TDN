use std::net::SocketAddr;

use crate::actor::prelude::{Addr, Message};
use crate::primitives::types::{BlockByte, EventID, GroupID, LevelPermissionByte, RPCParams};

use crate::traits::actor::RPCBridgeActor;

/// rpc request from local outside.
/// Params is rpc_session_id, group_id, RPCParams, and socket_addr,
/// if want to request other socket, use socket_addr, when receive is none.
#[derive(Clone)]
pub struct ReceiveLocalMessage(pub GroupID, pub usize, pub RPCParams, pub SocketAddr);

impl Message for ReceiveLocalMessage {
    type Result = ();
}

/// rpc response from local outside.
/// Params is rpc_session_id, group_id, RPCParams, and socket_addr,
#[derive(Clone)]
pub struct ReceiveLocalResponseMessage(pub GroupID, pub usize, pub Option<RPCParams>);

impl Message for ReceiveLocalResponseMessage {
    type Result = ();
}

/// rpc request from upper level group (send block for subscribed).
/// Params is rpc_session_id, group_id, Block Byte.
#[derive(Clone)]
pub struct ReceiveUpperMessage(pub GroupID, pub usize, pub BlockByte);

impl Message for ReceiveUpperMessage {
    type Result = ();
}

/// rpc response from upper level group (send block for subscribed).
/// Params is session_id, response.
#[derive(Clone)]
pub struct ReceiveUpperResponseMessage(pub GroupID, pub usize, pub Option<EventID>);

impl Message for ReceiveUpperResponseMessage {
    type Result = ();
}

/// rpc request from lower level group (send block get more security).
/// Params is rpc_session_id, group_id, Block Byte.
#[derive(Clone)]
pub struct ReceiveLowerMessage(pub GroupID, pub usize, pub BlockByte);

impl Message for ReceiveLowerMessage {
    type Result = ();
}

/// rpc response from lower level group (send block get more security).
/// Params is rpc_session_id, response
#[derive(Clone)]
pub struct ReceiveLowerResponseMessage(pub GroupID, pub usize, pub Option<EventID>);

impl Message for ReceiveLowerResponseMessage {
    type Result = ();
}

/// rpc level permission request.
/// Params is LevelPermissionByte.
#[derive(Clone)]
pub struct ReceiveLevelPermissionMessage(
    pub GroupID,
    pub usize,
    pub LevelPermissionByte,
    pub SocketAddr,
);

impl Message for ReceiveLevelPermissionMessage {
    type Result = ();
}

/// rpc level permission response.
/// Params is LevelPermissionByte.
#[derive(Clone)]
pub struct ReceiveLevelPermissionResponseMessage(pub GroupID, pub usize, pub bool);

impl Message for ReceiveLevelPermissionResponseMessage {
    type Result = ();
}

/// when rpc bridge actor start, need register addr to rpc actor
#[derive(Clone)]
pub struct RPCBridgeAddrMessage<B: RPCBridgeActor>(pub Addr<B>);

impl<B: RPCBridgeActor> Message for RPCBridgeAddrMessage<B> {
    type Result = ();
}
