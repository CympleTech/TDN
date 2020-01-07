use serde::de::DeserializeOwned;
use serde::ser::Serialize as SerializeOwned;
use serde_derive::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::fmt::{Debug, Formatter, Result as FmtResult};

use crate::error::Error;
use crate::p2p::PeerId as PeerAddr;

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
        sha.input(&s);
        let mut peer_bytes = [0u8; 32];
        peer_bytes.copy_from_slice(&sha.result()[..]);
        GroupId(peer_bytes)
    }

    pub fn from_hex(s: impl ToString) -> Result<GroupId, Error> {
        let s = s.to_string();
        if s.len() != 64 {
            return Err(Error::Hex);
        }

        let mut value = [0u8; 32];

        for i in 0..(s.len() / 2) {
            let res = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).map_err(|_e| Error::Hex)?;
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

/// This is the interface of the group in the entire network,
/// you can customize the implementation of these methods,
/// you can set the permission or size of the group
pub trait Group: Send + Debug {
    type JoinType: Clone + Send + Default + Debug + SerializeOwned + DeserializeOwned = ();
    type JoinResultType: Clone + Send + Default + Debug + SerializeOwned + DeserializeOwned = ();
    type LowerType: Clone + Send + Default + Debug + SerializeOwned + DeserializeOwned = Vec<u8>;
}
