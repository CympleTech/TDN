use std::net::SocketAddr;

use crate::actor::prelude::{Addr, Message};
use crate::primitives::types::{EventByte, GroupID, PeerAddr, PeerInfoByte};

use crate::traits::actor::P2PBridgeActor;

/// receive event message between p2p & bridge.
/// Params peerAddr, Event Byte.
#[derive(Clone)]
pub struct ReceiveEventMessage(pub GroupID, pub PeerAddr, pub EventByte);

impl Message for ReceiveEventMessage {
    type Result = ();
}

/// receive peer join between p2p & bridge.
/// Params is PeerAddr (p2p Node), Peer Join Info Byte.
#[derive(Clone)]
pub struct ReceivePeerJoinMessage(
    pub GroupID,
    pub PeerAddr,
    pub PeerInfoByte,
    pub Option<SocketAddr>,
);

impl Message for ReceivePeerJoinMessage {
    type Result = ();
}

/// peer join result when receive join request between p2p & bridge.
/// Params is PeerAddr (p2p Node), bool (join ok or not), help some peer addr.
#[derive(Clone)]
pub struct ReceivePeerJoinResultMessage(pub GroupID, pub PeerAddr, pub bool, pub Vec<PeerAddr>);

impl Message for ReceivePeerJoinResultMessage {
    type Result = ();
}

/// receive peer leave between p2p & bridge.
/// Params is PeerAddr (p2p Node), bool if is true, lost by all peers,
/// if false, only first lost by self lost.
#[derive(Clone)]
pub struct ReceivePeerLeaveMessage(pub GroupID, pub PeerAddr, pub bool);

impl Message for ReceivePeerLeaveMessage {
    type Result = ();
}

/// when p2p bridge actor start, need register addr to p2p actor
#[derive(Clone)]
pub struct P2PBridgeAddrMessage<B: P2PBridgeActor>(pub Addr<B>);

impl<B: P2PBridgeActor> Message for P2PBridgeAddrMessage<B> {
    type Result = ();
}
