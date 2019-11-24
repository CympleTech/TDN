use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::fmt::Debug;

use crate::p2p::PeerId;

#[derive(Debug, Clone, Default)]
pub struct GroupId;

/// This is the interface of the group in the entire network,
/// you can customize the implementation of these methods,
/// you can set the permission or size of the group
pub trait Group: Send {
    type JoinType: Clone + Send + Default + Debug + Serialize + DeserializeOwned;

    /// id: it will return group's id
    fn id(&self) -> &GroupId;

    /// directly add a peer to group.
    fn add(&mut self, peer_id: &PeerId);

    /// join: when peer join will call, happen before call consensus's peer_join,
    /// it has default implement if you want to handle it in consensus
    fn join(&mut self, _peer_id: PeerId, _data: Self::JoinType) -> bool {
        true
    }

    /// leave: when peer leave will call, happen before call consensus's peer_leave,
    /// it has default implement if you want to handle it in consensus
    fn leave(&mut self, _peer_id: &PeerId) -> bool {
        true
    }

    /// verify: check peer is verified by group permission,
    /// it has default implement if you want to handle it in consensus
    fn verify(&self, _peer_id: &PeerId) -> bool {
        true
    }
}
