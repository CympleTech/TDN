#![feature(vec_remove_item)]

use core::actor::prelude::{Actor, Addr};
use core::traits::propose::Message;
use core::traits::propose::Peer;

mod actor;
mod block;
mod event;
mod fixed_chain;
mod leader;
mod message;
mod store;
mod traits;

pub use actor::QuickPBFTActor;
pub use block::{Block, BlockID};
pub use event::Event;
pub use traits::QuickPBFTBridgeActor;
pub use traits::{
    HandleEventMessage, HandleEventResultMessage, HandlePeerInMessage, HandlePeerOutMessage,
};

pub fn quick_pbft_start<M: 'static + Message, P: 'static + Peer, A: QuickPBFTBridgeActor<M, P>>(
    pk: P::PublicKey,
    psk: P::PrivateKey,
    addr: Addr<A>,
    peers: Vec<P::PublicKey>,
) -> Addr<QuickPBFTActor<M, P, A>> {
    QuickPBFTActor::<M, P, A>::create(|ctx| {
        ctx.set_mailbox_capacity(100);
        QuickPBFTActor::load(pk, psk, addr, peers)
    })
}
