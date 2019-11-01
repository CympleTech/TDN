use crate::actor::prelude::*;
use crate::traits::message::rpc_message::*;

pub trait RPCBridgeActor<R = Context<Self>>
where
    Self: Clone
        + Actor<Context = R>
        + Handler<ReceiveLocalMessage>
        + Handler<ReceiveUpperMessage>
        + Handler<ReceiveLowerMessage>
        + Handler<ReceiveLevelPermissionMessage>
        + Handler<ReceiveLocalResponseMessage>
        + Handler<ReceiveUpperResponseMessage>
        + Handler<ReceiveLowerResponseMessage>
        + Handler<ReceiveLevelPermissionResponseMessage>,
    R: ActorContext
        + ToEnvelope<Self, ReceiveLocalMessage>
        + ToEnvelope<Self, ReceiveUpperMessage>
        + ToEnvelope<Self, ReceiveLowerMessage>
        + ToEnvelope<Self, ReceiveLevelPermissionMessage>
        + ToEnvelope<Self, ReceiveLocalResponseMessage>
        + ToEnvelope<Self, ReceiveUpperResponseMessage>
        + ToEnvelope<Self, ReceiveLowerResponseMessage>
        + ToEnvelope<Self, ReceiveLevelPermissionResponseMessage>,
{
}
