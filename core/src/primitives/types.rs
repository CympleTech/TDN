use jsonrpc_parse::Params;
use serde_derive::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::ops::Index;

use crate::crypto::hash::H256;
//use crate::crypto::hash::H512;
use crate::crypto::keypair::PublicKey;

pub const GROUP_ID_LENGTH: usize = 32;
pub type GroupID = H256;

pub const EVENT_ID_LENGTH: usize = 32;
pub type EventID = H256;

pub const APP_ID_LENGTH: usize = 32;
pub type AppID = H256;

pub type PeerAddr = PublicKey;
pub type RPCParams = Params;
pub type BlockByte = Vec<u8>;
pub type EventByte = Vec<u8>;
pub type PeerInfoByte = Vec<u8>;
pub type LevelPermissionByte = Vec<u8>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct App<'a> {
    id: AppID,
    symbol: &'a str,
    owner: PublicKey,
}

impl<'a> App<'a> {
    pub fn new(symbol: &'a str, owner: PublicKey) -> Self {
        let mut data = Vec::new();
        data.extend(bincode::serialize(&symbol).unwrap());
        data.extend(bincode::serialize(&owner).unwrap());
        let id = AppID::new(&data[..]);

        App { id, symbol, owner }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash, Eq)]
pub struct Binary(Vec<bool>);

impl Binary {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn new(vec: &Vec<bool>) -> Binary {
        Binary(vec.clone())
    }

    pub fn max(len: usize) -> Binary {
        let vec = vec![true; len];
        Binary(vec)
    }

    pub fn to_string(&self) -> String {
        let mut string = String::new();
        for i in &self.0 {
            string.push_str(match i {
                true => "1",
                false => "0",
            });
        }
        string
    }

    pub fn get_same_prefix(&self) -> Binary {
        let mut vec: Vec<bool> = Vec::new();
        let first_prefix: bool = self[0];
        for i in &self.0 {
            if i == &first_prefix {
                vec.push(first_prefix)
            } else {
                break;
            }
        }
        Binary(vec)
    }

    pub fn range(&self, start: usize, end: usize) -> Binary {
        let true_end = if self.len() < end { self.len() } else { end };

        let mut vec: Vec<bool> = Vec::new();
        for i in start..true_end {
            vec.push(self[i])
        }
        Binary(vec)
    }

    pub fn xor(&self, other: &Binary) -> Binary {
        let mut xor_: Vec<bool> = Vec::new();

        for i in 0..self.len() {
            xor_.push(self[i] ^ other[i])
        }

        Binary::new(&xor_)
    }
}

impl Index<usize> for Binary {
    type Output = bool;

    fn index(&self, index: usize) -> &bool {
        &self.0[index]
    }
}

impl Ord for Binary {
    fn cmp(&self, other: &Binary) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for Binary {
    fn partial_cmp(&self, other: &Binary) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Binary {
    fn eq(&self, other: &Binary) -> bool {
        self.0 == other.0
    }
}

impl Default for Binary {
    fn default() -> Binary {
        let vec = vec![true; 8];
        Binary(vec)
    }
}
