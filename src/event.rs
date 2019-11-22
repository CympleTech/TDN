use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::fmt::Debug;

use crate::primitives::types::EventID;

/// This is the interface of the Event in the entire network,
/// Event id's data structure is defined by teatree,
/// Events are the basic unit of flow in the network
pub trait Event: Clone + Send + Debug + Eq + Ord + Serialize + DeserializeOwned {
    /// get the event id, defined in teatree
    fn id(&self) -> &EventID;
}
