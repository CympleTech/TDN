use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::fmt::{Debug, Formatter, Result as FmtResult};

use crate::message::{GroupSendMessage, LayerSendMessage};
use crate::primitive::RpcParam;

/// Type: GroupId
#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct GroupId(pub [u8; 32]);

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
        let mut peer_bytes = [0u8; 32];
        peer_bytes.copy_from_slice(&sha.finalize()[..]);
        GroupId(peer_bytes)
    }

    pub fn from_hex(s: impl ToString) -> Result<GroupId, &'static str> {
        let s = s.to_string();
        if s.len() != 64 {
            return Err("Hex is invalid");
        }

        let mut value = [0u8; 32];

        for i in 0..(s.len() / 2) {
            let res =
                u8::from_str_radix(&s[2 * i..2 * i + 2], 16).map_err(|_e| "Hex is invalid")?;
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
    /// define join_data's type.
    type JoinType;
    /// define join_result_data's type.
    type JoinResultType;

    /// get the group id, defined in TDN
    fn id(&self) -> &GroupId;
}

/// Helper: this is the interface of the Peer in the network.
pub trait Peer {
    type PublicKey: Serialize + DeserializeOwned;
    type SecretKey: Serialize + DeserializeOwned;
    type Signature: Serialize + DeserializeOwned;

    fn sign(
        sk: &Self::SecretKey,
        msg: &Vec<u8>,
    ) -> Result<Self::Signature, Box<dyn std::error::Error>>;

    fn verify(pk: &Self::PublicKey, msg: &Vec<u8>, sign: &Self::Signature) -> bool;

    fn hex_public_key(pk: &Self::PublicKey) -> String;
}

/// Helper: this is the EventId in the network.
pub struct EventId(pub [u8; 64]);

/// Helper: this is the interface of the Event in the network.
pub trait Event: Clone + Send + Debug + Eq + Ord + Serialize + DeserializeOwned {
    /// get the event id, defined in TDN
    fn id(&self) -> &EventId;
}

/// Helper: this is the Event handle result in the network.
pub struct EventResult {
    pub rpcs: Vec<(String, RpcParam)>,
    pub groups: Vec<GroupSendMessage>,
    pub layers: Vec<LayerSendMessage>,
}
