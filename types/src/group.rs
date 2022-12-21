use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

use crate::message::RecvType;
use crate::primitives::{HandleResult, PeerId, Result};

/// Group Id bytes length.
pub const GROUP_BYTES_LENGTH: usize = 8;

/// Type: GroupId
pub type GroupId = u64;

/// This is random bytes lookup
const HASH_TABLE: [u8; 256] = [
    110, 116, 190, 7, 150, 121, 239, 74, 67, 16, 113, 25, 103, 91, 131, 227, 254, 214, 153, 180,
    182, 192, 137, 253, 171, 141, 215, 130, 76, 105, 66, 140, 87, 194, 143, 69, 36, 118, 41, 223,
    84, 205, 210, 224, 149, 8, 102, 157, 136, 10, 78, 233, 104, 99, 243, 114, 178, 81, 115, 127,
    132, 235, 160, 226, 170, 245, 44, 42, 169, 159, 228, 98, 189, 152, 48, 231, 172, 246, 181, 37,
    161, 1, 158, 122, 97, 52, 39, 5, 82, 128, 24, 217, 15, 47, 196, 31, 94, 85, 222, 14, 154, 73,
    6, 213, 123, 156, 109, 174, 9, 45, 144, 249, 165, 95, 72, 175, 77, 164, 75, 197, 242, 49, 55,
    58, 176, 124, 139, 135, 145, 129, 126, 202, 59, 111, 96, 195, 33, 200, 166, 117, 19, 208, 219,
    27, 65, 142, 252, 184, 54, 89, 71, 40, 185, 64, 79, 203, 241, 20, 17, 167, 29, 53, 50, 38, 206,
    230, 201, 43, 26, 238, 93, 32, 21, 92, 86, 28, 207, 248, 221, 106, 237, 83, 63, 138, 90, 198,
    187, 46, 148, 163, 51, 70, 244, 188, 12, 199, 80, 0, 147, 247, 168, 23, 22, 173, 56, 101, 204,
    107, 18, 177, 225, 232, 61, 186, 2, 211, 3, 30, 133, 236, 218, 151, 155, 108, 179, 112, 62,
    125, 13, 255, 120, 191, 212, 35, 229, 34, 11, 216, 88, 250, 183, 134, 251, 119, 193, 220, 234,
    162, 240, 100, 146, 60, 57, 68, 4, 209,
];

/// Hash to GroupId, this is not a cryptographically secure hash,
/// it uses Pearson hashing.
pub fn hash_to_group_id(bytes: &[u8]) -> GroupId {
    if bytes.len() == 0 {
        return 0;
    }

    let res: Vec<u8> = (0..GROUP_BYTES_LENGTH)
        .map(|byte| {
            let mut h = HASH_TABLE[(bytes[0] as usize + byte) % 256];
            for b in &bytes[1..] {
                h = HASH_TABLE[(h ^ b) as usize];
            }
            h
        })
        .collect();
    let mut fixed = [0u8; GROUP_BYTES_LENGTH];
    fixed.copy_from_slice(&res);
    GroupId::from_le_bytes(fixed)
}

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hash_to_group_id() {
        let test1 = "";
        let res = hash_to_group_id(test1.as_bytes());
        assert_eq!(0, res);

        let test2 = [0u8];
        let res = hash_to_group_id(&test2);
        assert_eq!(5399668163522491502, res);

        let test3 = "hello, world";
        let res = hash_to_group_id(test3.as_bytes());
        assert_eq!(908348025392027244, res);

        let test4 = "hello, world!";
        let res = hash_to_group_id(test4.as_bytes());
        assert_eq!(629292594133124598, res);

        let test5 = "tdnnnnnnnnnnnnnnnnnnnnnnnnnnnn";
        let res = hash_to_group_id(test5.as_bytes());
        assert_eq!(11309050355555053027, res);
    }
}
