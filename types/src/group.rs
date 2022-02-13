use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

use crate::message::RecvType;
use crate::primitives::{HandleResult, PeerId, Result};

/// Group Id bytes length.
pub const GROUP_BYTES_LENGTH: usize = 4;

/// Type: GroupId
pub type GroupId = u32;

/// Helper: this is the interface of the Group in the network.
/// Group id's data structure is defined by TDN,
pub trait Group {
    /// get the group id, defined in TDN
    fn id(&self) -> &GroupId;

    /// guard if the peer is valid.
    fn guard(&self, addr: &PeerId) -> bool;

    /// when receive group message, handle it, and return HandleResult.
    fn handle(&mut self, msg: RecvType) -> Result<HandleResult>;
}

/// Helper: this is the interface of the Peer in the network.
pub trait Peer {
    type PublicKey: Serialize + DeserializeOwned + Eq + std::hash::Hash + Clone;
    type SecretKey: Serialize + DeserializeOwned;
    type Signature: Serialize + DeserializeOwned;

    fn sign(sk: &Self::SecretKey, msg: &Vec<u8>) -> Result<Self::Signature>;

    fn verify(pk: &Self::PublicKey, msg: &Vec<u8>, sign: &Self::Signature) -> bool;
}

/// Helper: this is the EventId in the network.
#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct EventId(pub [u8; 32]);

impl EventId {
    pub fn from_hex(s: impl ToString) -> Result<EventId> {
        let s = s.to_string();
        if s.len() != 64 {
            return Err(anyhow::anyhow!("Hex is invalid"));
        }

        let mut value = [0u8; 32];

        for i in 0..(s.len() / 2) {
            let res = u8::from_str_radix(&s[2 * i..2 * i + 2], 16)?;
            value[i] = res;
        }

        Ok(EventId(value))
    }

    pub fn to_hex(&self) -> String {
        let mut hex = String::new();
        hex.extend(self.0.iter().map(|byte| format!("{:02x?}", byte)));
        hex
    }
}

/// Helper: this is the interface of the Event in the network.
pub trait Event: Clone + Send + Debug + Eq + Ord + Serialize + DeserializeOwned {
    /// get the event id, defined in TDN
    fn id(&self) -> &EventId;
}
