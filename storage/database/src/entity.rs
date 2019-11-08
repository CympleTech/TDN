use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::convert::AsRef;

use crate::actor::prelude::Message;

pub trait Entity: Serialize + DeserializeOwned {
    type Key: AsRef<[u8]>;

    fn key(&self) -> Self::Key;
}

/// read message
#[derive(Clone)]
pub struct EntityRead<E: 'static + Entity>(pub E::Key);

impl<E: 'static + Entity> Message for EntityRead<E> {
    type Result = Result<E, ()>;
}

/// write message
#[derive(Clone)]
pub struct EntityWrite<E: Entity>(pub E);

impl<E: Entity> Message for EntityWrite<E> {
    type Result = ();
}

/// delete message
#[derive(Clone)]
pub struct EntityDelete<E: 'static + Entity>(pub E::Key);

impl<E: 'static + Entity> Message for EntityDelete<E> {
    type Result = Result<E, ()>;
}
