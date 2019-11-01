#![feature(vec_remove_item)]

mod actor;
mod message;

pub use actor::GossipActor;
pub use message::{Gossip, GossipConfirm, GossipMessage, GossipNew, GossipP2P, GossipPeerLeave};
