use serde_derive::{Deserialize, Serialize};
use std::cmp::Ordering;

use core::primitives::types::EventID;
use core::traits::propose::Event as EventTrait;
use core::traits::propose::Message as MessageTrait;
use core::traits::propose::Peer as PeerTrait;

use crate::block::Block;
use crate::message::Message;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Event<M: MessageTrait, P: PeerTrait> {
    id: EventID,
    pub message: Message<M, P>,
    pub creator: P::PublicKey,
    signature: P::Signature,
}

impl<M: MessageTrait, P: PeerTrait> Event<M, P> {
    pub fn new_from_self_message(
        creator: P::PublicKey,
        message: Message<M, P>,
        psk: &P::PrivateKey,
    ) -> Self {
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

    pub fn message(&self) -> Option<&M> {
        if let Message::Tx(m) = &self.message {
            Some(&m)
        } else {
            None
        }
    }

    pub fn block(&self) -> Option<&Block<M, P>> {
        if let Message::Block(block) = &self.message {
            Some(block)
        } else {
            None
        }
    }

    pub fn new_from_message(creator: P::PublicKey, m: M, psk: &P::PrivateKey) -> Self {
        let message = Message::Tx(m);
        Self::new_from_self_message(creator, message, psk)
    }

    pub fn is_effective(&self) -> bool {
        self.message.is_effective()
    }
}

impl<M: MessageTrait, P: 'static + PeerTrait> Eq for Event<M, P> {}

impl<M: MessageTrait, P: 'static + PeerTrait> Ord for Event<M, P> {
    fn cmp(&self, other: &Event<M, P>) -> Ordering {
        self.id().cmp(other.id())
    }
}

impl<M: MessageTrait, P: 'static + PeerTrait> PartialOrd for Event<M, P> {
    fn partial_cmp(&self, other: &Event<M, P>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<M: MessageTrait, P: 'static + PeerTrait> PartialEq for Event<M, P> {
    fn eq(&self, other: &Event<M, P>) -> bool {
        self.id() == other.id()
    }
}

impl<M: MessageTrait, P: 'static + PeerTrait> EventTrait for Event<M, P> {
    fn id(&self) -> &EventID {
        &self.id
    }
}
