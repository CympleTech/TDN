use crate::actor::prelude::*;
use crate::traits::message::bridge_message::*;

pub trait BridgeActor<R = Context<Self>>
where
    Self: Actor<Context = R>
        + Handler<EventMessage>
        + Handler<PeerJoinMessage>
        + Handler<PeerJoinResultMessage>
        + Handler<PeerLeaveMessage>
        + Handler<LocalMessage>
        + Handler<UpperMessage>
        + Handler<LowerMessage>
        + Handler<LevelPermissionMessage>
        + Handler<LocalResponseMessage>
        + Handler<UpperResponseMessage>
        + Handler<LowerResponseMessage>
        + Handler<LevelPermissionResponseMessage>,

    R: ActorContext
        + ToEnvelope<Self, EventMessage>
        + ToEnvelope<Self, PeerJoinMessage>
        + ToEnvelope<Self, PeerJoinResultMessage>
        + ToEnvelope<Self, PeerLeaveMessage>
        + ToEnvelope<Self, LocalMessage>
        + ToEnvelope<Self, UpperMessage>
        + ToEnvelope<Self, LowerMessage>
        + ToEnvelope<Self, LevelPermissionMessage>
        + ToEnvelope<Self, LocalResponseMessage>
        + ToEnvelope<Self, UpperResponseMessage>
        + ToEnvelope<Self, LowerResponseMessage>
        + ToEnvelope<Self, LevelPermissionResponseMessage>,
{
}
