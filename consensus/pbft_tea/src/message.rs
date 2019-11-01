use serde_derive::{Deserialize, Serialize};

use teatree::traits::propose::Message as MessageTrait;
use teatree::traits::propose::Peer as PeerTrait;

use crate::block::{Block, BlockID};

#[serde(bound = "")]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Message<M: MessageTrait, P: PeerTrait> {
    Tx(M),
    Block(Block<M, P>),
    Verify(BlockID),
    Commit(BlockID),
    Sync(u64),
    BlockReq(BlockID),
    BlockRes(Option<Block<M, P>>),
    Leader(P::PublicKey, u64),
    HeartBeat,
}

impl<M: MessageTrait, P: PeerTrait> Message<M, P> {
    pub fn is_effective(&self) -> bool {
        match self {
            Message::Tx(_) => true,
            _ => false,
        }
    }
}
