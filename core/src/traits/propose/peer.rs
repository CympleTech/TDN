use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use serde_derive::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::marker::Send;
use std::marker::Sync;

use crate::primitives::types::Binary;
use crate::primitives::types::{GroupID, PeerAddr};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PeerID<P: Peer> {
    pub group: GroupID,
    pub pk: P::PublicKey,
    pub addr: PeerAddr,
}

impl<P: Peer> PeerID<P> {
    pub fn new(group: GroupID, pk: P::PublicKey, addr: PeerAddr) -> Self {
        PeerID { group, pk, addr }
    }

    pub fn group(&self) -> &GroupID {
        &self.group
    }

    pub fn pk(&self) -> &P::PublicKey {
        &self.pk
    }

    pub fn addr(&self) -> &PeerAddr {
        &self.addr
    }

    pub fn verify(&self, meta_data: &Vec<u8>, signature: &P::Signature) -> bool {
        P::verify(&self.pk, meta_data, signature)
    }

    pub fn new_by_pk(&self, pk: &P::PublicKey) -> Self {
        PeerID {
            group: self.group.clone(),
            pk: pk.clone(),
            addr: self.addr.clone(),
        }
    }
}

pub trait Peer: Default + Clone + Debug + Sync + Send + Serialize + DeserializeOwned {
    type PrivateKey: Clone
        + Debug
        + Eq
        + Ord
        + Sync
        + Send
        + Serialize
        + DeserializeOwned
        + Display
        + From<String>;
    type PublicKey: Hash
        + Default
        + Clone
        + Debug
        + Eq
        + Ord
        + Sync
        + Send
        + Serialize
        + DeserializeOwned
        + Display
        + From<String>;

    type Signature: Clone
        + Default
        + Debug
        + Eq
        + Ord
        + Sync
        + Send
        + Serialize
        + DeserializeOwned
        + Display
        + From<String>;

    const PRIVATE_KEY_LENGTH: usize;
    const PUBLIC_KEY_LENGTH: usize;
    const SIGNATURE_KEY_LENGTH: usize;

    fn generate() -> (Self::PublicKey, Self::PrivateKey);

    fn pk(&self) -> &Self::PublicKey;

    fn sign(psk: &Self::PrivateKey, data: &Vec<u8>) -> Self::Signature;

    fn verify(pk: &Self::PublicKey, data: &Vec<u8>, signature: &Self::Signature) -> bool;

    fn binary(&self) -> Binary {
        let self_bytes = bincode::serialize(self.pk()).unwrap();
        let mut vec: Vec<bool> = Vec::new();
        for i in 0..self_bytes.len() {
            let str_a = format!("{:>08b}", self_bytes[i]);
            for i in str_a.chars() {
                vec.push(match i {
                    '1' => true,
                    _ => false,
                });
            }
        }

        Binary::new(&vec)
    }

    fn xor(&self, other_peer: &impl Peer) -> Binary {
        self.binary().xor(&other_peer.binary())
    }

    fn public_key_from_bytes(bytes: &[u8]) -> Option<Self::PublicKey>;

    fn public_key_to_bytes(pk: &Self::PublicKey) -> Vec<u8>;

    fn private_key_from_bytes(bytes: &[u8]) -> Option<Self::PrivateKey>;

    fn private_key_to_bytes(psk: &Self::PrivateKey) -> Vec<u8>;
}
