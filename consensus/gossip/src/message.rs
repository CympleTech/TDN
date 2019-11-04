use serde_derive::{Deserialize, Serialize};

use core::actor::prelude::*;
use core::primitives::types::EventID;
use core::traits::propose::Peer;

use crate::actor::SeeMap;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub struct Gossip<P: Peer>(pub EventID, pub SeeMap<P>);

#[derive(Clone)]
pub struct GossipMessage<P: Peer>(pub P::PublicKey, pub Gossip<P>);

impl<P: Peer> Message for GossipMessage<P> {
    type Result = ();
}

#[derive(Clone)]
pub struct GossipPeerLeave<P: Peer>(pub P::PublicKey);

impl<P: Peer> Message for GossipPeerLeave<P> {
    type Result = ();
}

#[derive(Clone)]
pub struct GossipNew<P: Peer>(pub EventID, pub Vec<P::PublicKey>);

impl<P: Peer> Message for GossipNew<P> {
    type Result = ();
}

#[derive(Clone)]
pub struct GossipP2P<P: Peer>(pub P::PublicKey, pub P::PublicKey, pub GossipMessage<P>);

impl<P: Peer> GossipP2P<P> {
    pub fn event_id(&self) -> &EventID {
        &(((self.2).1).0)
    }
}

impl<P: Peer> Message for GossipP2P<P> {
    type Result = ();
}

#[derive(Clone)]
pub struct GossipConfirm<P: Peer>(pub P::PublicKey, pub EventID, pub SeeMap<P>);

impl<P: Peer> Message for GossipConfirm<P> {
    type Result = ();
}
