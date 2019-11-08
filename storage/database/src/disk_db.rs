use sled::Db;
use std::convert::AsRef;
use std::path::PathBuf;

use crate::primitives::functions::get_default_storage_path;

use super::entity::Entity;

pub struct DiskDatabase {
    db: Db,
}

impl DiskDatabase {
    pub fn new(path: Option<PathBuf>) -> Self {
        let path = if path.is_none() {
            let mut path = get_default_storage_path();
            path.push("storage");
            path
        } else {
            path.unwrap()
        };

        DiskDatabase {
            db: Db::start_default(path).unwrap(),
        }
    }

    pub fn write(&self, key: impl AsRef<[u8]>, value: Vec<u8>) {
        let _ = self.db.set(key, value);
        let _ = self.db.flush();
    }

    pub fn read(&self, key: &impl AsRef<[u8]>) -> Result<Vec<u8>, ()> {
        if let Ok(Some(data)) = self.db.get(key) {
            Ok(data.to_vec())
        } else {
            Err(())
        }
    }

    pub fn delete(&self, key: &impl AsRef<[u8]>) -> Result<Vec<u8>, ()> {
        if let Ok(Some(data)) = self.db.del(key) {
            Ok(data.to_vec())
        } else {
            Err(())
        }
    }

    pub fn write_entity<E: Entity>(&self, entity: E) {
        self.write(entity.key(), bincode::serialize(&entity).unwrap())
    }

    pub fn read_entity<E: Entity>(&self, key: E::Key) -> Result<E, ()> {
        if let Ok(data) = self.read(&key) {
            let e = bincode::deserialize(&data);
            if e.is_ok() {
                Ok(e.unwrap())
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    pub fn delete_entity<E: Entity>(&self, key: E::Key) -> Result<E, ()> {
        if let Ok(data) = self.delete(&key) {
            let e = bincode::deserialize(&data);
            if e.is_ok() {
                Ok(e.unwrap())
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}
