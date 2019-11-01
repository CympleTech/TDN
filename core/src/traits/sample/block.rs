use serde_derive::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter, Result};
use std::slice::Iter;
use time::Timespec;

use crate::crypto::hash::H256;
use crate::traits::propose::{Event, Peer};

pub type BlockID = H256;

#[serde(bound = "")]
#[derive(Clone, Serialize, Deserialize)]
pub struct Block<P: Peer, E: Event> {
    id: BlockID,
    events: Vec<E>,
    blocker: P::PublicKey,
    signature: P::Signature,
    timestamp: i64,
    prev: BlockID,
    height: u64,
    //merkle: H256,
}

impl<P: Peer, E: Event> Block<P, E> {
    pub fn id(&self) -> &BlockID {
        &self.id
    }

    pub fn blocker(&self) -> &P::PublicKey {
        &self.blocker
    }

    pub fn height(&self) -> u64 {
        self.height
    }

    pub fn new(
        blocker: P::PublicKey,
        psk: &P::PrivateKey,
        events: Vec<E>,
        prev: BlockID,
        height: u64,
    ) -> Self {
        let mut data = Vec::new();

        data.extend(bincode::serialize(&events).unwrap());
        data.extend(bincode::serialize(&blocker).unwrap());
        data.extend(bincode::serialize(&prev).unwrap());
        data.extend(bincode::serialize(&height).unwrap());

        let mut hash_data = data.clone();

        let signature = P::sign(psk, &data);
        hash_data.extend(bincode::serialize(&signature).unwrap());

        let timestamp = time::now_utc().to_timespec().sec;
        hash_data.extend(bincode::serialize(&timestamp).unwrap());

        let id = BlockID::new(&hash_data[..]);

        Self {
            id,
            events,
            blocker,
            signature,
            timestamp,
            prev,
            height,
        }
    }

    pub fn iter(&self) -> Iter<E> {
        self.events.iter()
    }

    pub fn created_time(&self) -> Timespec {
        Timespec::new(self.timestamp, 0)
    }

    pub fn previous(&self) -> &BlockID {
        &self.prev
    }
}

impl<P: Peer, E: Event> Debug for Block<P, E> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "ID: {}, Creator: {}", self.id(), self.blocker)
    }
}
