use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::fmt::Debug;

use crate::primitives::types::{GroupID, PeerAddr};

use super::peer::Peer;

/// This is the interface of the group in the entire network,
/// you can customize the implementation of these methods,
/// you can set the permission or size of the group
pub trait Group<P: Peer> {
    type JoinType: Clone + Send + Default + Debug + Serialize + DeserializeOwned;

    /// id: it will return group's id
    fn id(&self) -> &GroupID;

    /// join: when peer join will call, happen before call consensus's peer_join,
    /// it has default implement if you want to handle it in consensus
    fn join(&mut self, _data: Self::JoinType, _peer_addr: PeerAddr) -> bool {
        true
    }

    /// leave: when peer leave will call, happen before call consensus's peer_leave,
    /// it has default implement if you want to handle it in consensus
    fn leave(&mut self, _pk: &PeerAddr) -> bool {
        true
    }

    /// verify: check peer is verified by group permission,
    /// it has default implement if you want to handle it in consensus
    fn verify(&self, _pk: &P::PublicKey) -> bool {
        true
    }

    /// sync_peers: help peer build his group, it has default implement
    fn help_sync_peers(&self, _pk: &P::PublicKey) -> Vec<PeerAddr> {
        Vec::new()
    }

    fn add_sync_peers(&mut self, _pk: &P::PublicKey, _peer_addr: PeerAddr) {}
}
