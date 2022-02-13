use serde::{de::DeserializeOwned, Serialize};

use crate::primitives::Result;

pub trait Storage: 'static + Send + Sync {
    type Key: 'static + Send + Sync;

    fn read<T: Serialize + DeserializeOwned>(&self, key: &Self::Key) -> Option<T>;

    fn write<T: Serialize + DeserializeOwned>(&self, key: &Self::Key, value: &T) -> Result<()>;

    fn update<T: Serialize + DeserializeOwned>(&self, key: &Self::Key, value: &T) -> Result<()>;

    fn delete<T: Serialize + DeserializeOwned>(&self, key: &Self::Key) -> Result<T>;

    fn keystore(&self, _key: &Self::Key, _passed: &[u8], _secret: &[u8]) -> Result<()> {
        Err(anyhow::anyhow!("unimplemented"))
    }
}
