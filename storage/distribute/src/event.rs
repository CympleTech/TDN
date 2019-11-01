use serde_derive::{Deserialize, Serialize};
use std::cmp::Ordering;

use teatree::crypto::keypair::{PrivateKey, PublicKey, Signature};
use teatree::primitives::types::EventID;
use teatree::traits::propose::Event as EventTrait;

use crate::message::ID;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) enum EventType {
    Read(PublicKey, ID),
    Write(PublicKey, ID, Vec<u8>),
    Drop(PublicKey, Signature, ID),
    ReadResult(PublicKey, ID, Vec<u8>),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct Event {
    id: EventID,
    creator: PublicKey,
    signature: Signature,
    pub event_type: EventType,
}

impl Event {
    fn new(event_type: EventType, pk: &PublicKey, psk: &PrivateKey) -> Self {
        let creator = pk.clone();
        let mut data = Vec::new();

        data.extend(bincode::serialize(&event_type).unwrap());
        data.extend(bincode::serialize(pk).unwrap());

        let mut hash_data = data.clone();

        let signature = psk.sign_bytes(&data);
        hash_data.extend(bincode::serialize(&signature).unwrap());

        let id = EventID::new(&hash_data[..]);

        Event {
            id,
            creator,
            signature,
            event_type,
        }
    }

    pub fn new_read(id: ID, pk: &PublicKey, psk: &PrivateKey) -> Self {
        Event::new(EventType::Read(pk.clone(), id), pk, psk)
    }

    pub fn new_write(id: ID, data: Vec<u8>, pk: &PublicKey, psk: &PrivateKey) -> Self {
        Event::new(EventType::Write(pk.clone(), id, data), pk, psk)
    }

    pub fn new_drop(id: ID, pk: &PublicKey, psk: &PrivateKey) -> Self {
        //let signature = psk.sign()
        Event::new(EventType::Drop(pk.clone(), Default::default(), id), pk, psk) //TODO Drop Signature
    }

    pub fn new_read_result(
        tpk: PublicKey,
        id: ID,
        data: Vec<u8>,
        pk: &PublicKey,
        psk: &PrivateKey,
    ) -> Self {
        Event::new(EventType::ReadResult(tpk, id, data), pk, psk)
    }
}

impl EventTrait for Event {
    fn id(&self) -> &EventID {
        &self.id
    }
}

impl Eq for Event {}

impl Ord for Event {
    fn cmp(&self, other: &Event) -> Ordering {
        self.id().cmp(other.id())
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Event) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Event) -> bool {
        self.id() == other.id()
    }
}
