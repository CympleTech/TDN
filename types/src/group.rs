use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::fmt::{Debug, Formatter, Result as FmtResult};

use crate::message::RecvType;
use crate::primitive::{new_io_error, HandleResult, PeerAddr, Result};

pub const GROUP_LENGTH: usize = 32;

/// Type: GroupId
#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct GroupId(pub [u8; GROUP_LENGTH]);

impl GroupId {
    pub fn short_show(&self) -> String {
        let mut hex = String::new();
        hex.extend(self.0.iter().map(|byte| format!("{:02x?}", byte)));
        let mut new_hex = String::new();
        new_hex.push_str("0x");
        new_hex.push_str(&hex[0..4]);
        new_hex.push_str("...");
        new_hex.push_str(&hex[hex.len() - 5..]);
        new_hex
    }

    pub fn from_symbol(s: impl ToString) -> GroupId {
        let s = s.to_string();
        let mut sha = Sha3_256::new();
        sha.update(&s);
        let mut peer_bytes = [0u8; GROUP_LENGTH];
        peer_bytes.copy_from_slice(&sha.finalize()[..]);
        GroupId(peer_bytes)
    }

    pub fn from_hex(s: impl ToString) -> Result<GroupId> {
        let s = s.to_string();
        if s.len() != GROUP_LENGTH * 2 {
            return Err(new_io_error("Hex is invalid"));
        }

        let mut value = [0u8; GROUP_LENGTH];

        for i in 0..(s.len() / 2) {
            let res = u8::from_str_radix(&s[2 * i..2 * i + 2], 16)
                .map_err(|_e| new_io_error("Hex is invalid"))?;
            value[i] = res;
        }

        Ok(GroupId(value))
    }

    pub fn to_hex(&self) -> String {
        let mut hex = String::new();
        hex.extend(self.0.iter().map(|byte| format!("{:02x?}", byte)));
        hex
    }
}

impl Debug for GroupId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let mut hex = String::new();
        hex.extend(self.0.iter().map(|byte| format!("{:02x?}", byte)));
        write!(f, "0x{}", hex)
    }
}

/// Helper: this is the interface of the Group in the network.
/// Group id's data structure is defined by TDN,
pub trait Group {
    /// get the group id, defined in TDN
    fn id(&self) -> &GroupId;

    /// guard if the peer is valid.
    fn guard(&self, addr: &PeerAddr) -> bool;

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
            return Err(new_io_error("Hex is invalid"));
        }

        let mut value = [0u8; 32];

        for i in 0..(s.len() / 2) {
            let res = u8::from_str_radix(&s[2 * i..2 * i + 2], 16)
                .map_err(|_e| new_io_error("Hex is invalid"))?;
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
