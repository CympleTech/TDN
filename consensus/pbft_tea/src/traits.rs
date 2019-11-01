use teatree::actor::prelude::{Actor, ActorContext, Context, Handler, Message, ToEnvelope};
use teatree::traits::propose::Message as MessageTrait;
use teatree::traits::propose::Peer;

use crate::block::Block;
use crate::event::Event;

#[derive(Clone)]
pub struct HandleEventMessage<M: MessageTrait, P: Peer>(pub P::PublicKey, pub Event<M, P>);

impl<M: MessageTrait, P: Peer> Message for HandleEventMessage<M, P> {
    type Result = ();
}

#[derive(Clone)]
pub struct HandlePeerInMessage<P: Peer>(pub P::PublicKey);

impl<P: Peer> Message for HandlePeerInMessage<P> {
    type Result = ();
}

#[derive(Clone)]
pub struct HandlePeerOutMessage<P: Peer>(pub P::PublicKey);

impl<P: Peer> Message for HandlePeerOutMessage<P> {
    type Result = ();
}

#[derive(Clone)]
pub struct HandleEventResultMessage<M: MessageTrait, P: Peer>(
    pub Vec<(P::PublicKey, Event<M, P>)>,
    pub Option<Block<M, P>>,
);

impl<M: MessageTrait, P: Peer> Message for HandleEventResultMessage<M, P> {
    type Result = ();
}

pub trait QuickPBFTBridgeActor<M, P, R = Context<Self>>
where
    M: MessageTrait,
    P: Peer,
    Self: Clone + Actor<Context = R> + Handler<HandleEventResultMessage<M, P>>,
    R: ActorContext + ToEnvelope<Self, HandleEventResultMessage<M, P>>,
{
}
