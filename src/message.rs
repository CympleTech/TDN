use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::fmt::Debug;

pub trait Message: Clone + Send + Debug + Serialize + DeserializeOwned {}
