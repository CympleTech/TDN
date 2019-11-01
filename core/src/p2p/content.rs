use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::crypto::keypair::PublicKey;
use crate::primitives::types::{EventByte, PeerInfoByte};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(bound = "")]
pub enum P2PContent {
    HeartBeat,
    HeartBeatOk,
    DHT(Vec<(PublicKey, SocketAddr)>),
    Hole(PublicKey, SocketAddr),
    HolePunching,
    HolePunchingOk,
    None,

    /// need send to network bridge - PeerJoin
    Leave,

    /// need send to network bridge - PeerLeave
    Join(PeerInfoByte),

    /// need send to network bridge - Event
    Event(EventByte),
}
