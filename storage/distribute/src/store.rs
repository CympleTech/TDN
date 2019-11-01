use serde_derive::{Deserialize, Serialize};
use teatree::storage::Entity;

use crate::message::ID;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct DataStore(pub ID, pub Vec<u8>);

impl Entity for DataStore {
    type Key = ID;

    fn key(&self) -> Self::Key {
        self.0.clone()
    }
}
