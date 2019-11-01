use serde_derive::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::primitives::types::EventID;
use crate::traits::propose::Event as EventTrait;
use crate::traits::propose::Message;
use crate::traits::propose::Peer;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Event<M: Message, P: Peer> {
    id: EventID,
    message: M,
    creator: P::PublicKey,
    signature: P::Signature,
}

impl<M: Message, P: Peer> Event<M, P> {
    pub fn new(creator: P::PublicKey, message: M, psk: &P::PrivateKey) -> Self {
        let mut data = Vec::new();

        data.extend(bincode::serialize(&message).unwrap());
        data.extend(bincode::serialize(&creator).unwrap());

        let mut hash_data = data.clone();

        let signature = P::sign(psk, &data);
        hash_data.extend(bincode::serialize(&signature).unwrap());

        let id = EventID::new(&hash_data[..]);

        Self {
            id,
            message,
            creator,
            signature,
        }
    }

    pub fn creator(&self) -> &P::PublicKey {
        &self.creator
    }

    pub fn message(&self) -> &M {
        &self.message
    }

    pub fn verify(&self) -> bool {
        let mut data = Vec::new();

        data.extend(bincode::serialize(&self.message).unwrap());
        data.extend(bincode::serialize(&self.creator).unwrap());

        if !P::verify(&self.creator, &data, &self.signature) {
            return false;
        }

        let mut hash_data = data.clone();
        hash_data.extend(bincode::serialize(&self.signature).unwrap());

        EventID::new(&hash_data[..]) == self.id
    }
}

impl<M: Message, P: Peer> Eq for Event<M, P> {}

impl<M: Message, P: Peer> Ord for Event<M, P> {
    fn cmp(&self, other: &Event<M, P>) -> Ordering {
        self.id().cmp(other.id())
    }
}

impl<M: Message, P: Peer> PartialOrd for Event<M, P> {
    fn partial_cmp(&self, other: &Event<M, P>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<M: Message, P: Peer> PartialEq for Event<M, P> {
    fn eq(&self, other: &Event<M, P>) -> bool {
        self.id() == other.id()
    }
}

impl<M: Message, P: Peer> EventTrait for Event<M, P> {
    fn id(&self) -> &EventID {
        &self.id
    }
}
