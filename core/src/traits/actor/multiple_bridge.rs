use crate::actor::prelude::*;
use crate::traits::message::multiple_bridge_message::*;

pub trait MultipleBridgeActor<R = Context<Self>>
where
    Self: Actor<Context = R>
        + Handler<MultipleEventMessage>
        + Handler<MultiplePeerJoinMessage>
        + Handler<MultiplePeerJoinResultMessage>
        + Handler<MultiplePeerLeaveMessage>
        + Handler<MultipleLocalMessage>
        + Handler<MultipleUpperMessage>
        + Handler<MultipleLowerMessage>
        + Handler<MultipleLevelPermissionMessage>
        + Handler<MultipleLocalResponseMessage>
        + Handler<MultipleUpperResponseMessage>
        + Handler<MultipleLowerResponseMessage>
        + Handler<MultipleLevelPermissionResponseMessage>,

    R: ActorContext
        + ToEnvelope<Self, MultipleEventMessage>
        + ToEnvelope<Self, MultiplePeerJoinMessage>
        + ToEnvelope<Self, MultiplePeerJoinResultMessage>
        + ToEnvelope<Self, MultiplePeerLeaveMessage>
        + ToEnvelope<Self, MultipleLocalMessage>
        + ToEnvelope<Self, MultipleUpperMessage>
        + ToEnvelope<Self, MultipleLowerMessage>
        + ToEnvelope<Self, MultipleLevelPermissionMessage>
        + ToEnvelope<Self, MultipleLocalResponseMessage>
        + ToEnvelope<Self, MultipleUpperResponseMessage>
        + ToEnvelope<Self, MultipleLowerResponseMessage>
        + ToEnvelope<Self, MultipleLevelPermissionResponseMessage>,
{
}
