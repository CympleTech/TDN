use serde_derive::{Deserialize, Serialize};

use core::primitives::types::EventID;
use core::storage::Entity;
use core::traits::propose::{Event as EventTrait, Message as MessageTrait, Peer};

use super::block::{Block, BlockID};
use super::event::Event;

#[derive(Serialize, Deserialize, Clone)]
pub struct ChainStore<P: 'static + Peer>(pub P::PublicKey, pub Vec<BlockID>);

impl<P: Peer> Entity for ChainStore<P> {
    type Key = String;

    fn key(&self) -> Self::Key {
        self.0.to_string()
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(bound = "")]
pub struct EventStore<M: MessageTrait, P: Peer>(pub Event<M, P>);

impl<M: MessageTrait, P: 'static + Peer> Entity for EventStore<M, P> {
    type Key = EventID;

    fn key(&self) -> Self::Key {
        self.0.id().clone()
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(bound = "")]
pub struct BlockStore<M: MessageTrait, P: Peer>(pub Block<M, P>);

impl<M: MessageTrait, P: Peer> Entity for BlockStore<M, P> {
    type Key = BlockID;

    fn key(&self) -> Self::Key {
        self.0.id().clone()
    }
}
