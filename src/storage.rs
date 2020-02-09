use async_std::fs;
use async_std::io::Result as AsyncResult;
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;

use crate::primitive::DEFAULT_STORAGE_DIR;

pub async fn read_local_file(name: &str) -> AsyncResult<Vec<u8>> {
    let mut path = DEFAULT_STORAGE_DIR.clone();
    path.push(name);
    fs::read(path).await
}

pub async fn read_string_local_file(name: &str) -> AsyncResult<String> {
    let mut path = DEFAULT_STORAGE_DIR.clone();
    path.push(name);
    fs::read_to_string(path).await
}

pub async fn write_local_file(name: &str, data: &[u8]) -> AsyncResult<()> {
    let mut path = DEFAULT_STORAGE_DIR.clone();
    path.push(name);
    fs::write(path, data).await
}

pub async fn remove_local_file(name: &str) -> AsyncResult<()> {
    let mut path = DEFAULT_STORAGE_DIR.clone();
    path.push(name);
    fs::remove_file(path).await
}

pub async fn read_absolute_file(path: &PathBuf) -> AsyncResult<Vec<u8>> {
    fs::read(path).await
}

pub async fn read_string_absolute_file(path: &PathBuf) -> AsyncResult<String> {
    fs::read_to_string(path).await
}

pub async fn write_absolute_file(path: &PathBuf, data: &[u8]) -> AsyncResult<()> {
    fs::write(path, data).await
}

pub async fn remove_absolute_file(path: &PathBuf) -> AsyncResult<()> {
    fs::remove_file(path).await
}

pub struct LocalDB {
    tree: sled::Db,
}

impl LocalDB {
    pub fn open(name: &str) -> Result<LocalDB, ()> {
        let mut path = DEFAULT_STORAGE_DIR.clone();
        path.push(name);
        let tree = sled::open(path).map_err(|_e| ())?;
        Ok(LocalDB { tree })
    }

    pub fn open_absolute(path: &PathBuf) -> Result<LocalDB, ()> {
        let tree = sled::open(path).map_err(|_e| ())?;
        Ok(LocalDB { tree })
    }

    pub fn read<T: Serialize + DeserializeOwned>(&self, k: &Vec<u8>) -> Option<T> {
        self.tree
            .get(k)
            .ok()
            .map(|v| v)
            .flatten()
            .map(|v| bincode::deserialize(&v).ok())
            .flatten()
    }

    pub fn write<T: Serialize + DeserializeOwned>(&self, k: Vec<u8>, t: &T) -> Result<(), ()> {
        bincode::serialize(&t).map_err(|_e| ()).and_then(|bytes| {
            self.tree
                .insert(k, bytes)
                .map_err(|_e| ())
                .and_then(|_| self.flush())
        })
    }

    pub fn update<T: Serialize + DeserializeOwned>(&self, k: Vec<u8>, t: &T) -> Result<(), ()> {
        bincode::serialize(&t).map_err(|_e| ()).and_then(|bytes| {
            let old = self.tree.get(&k).ok().map(|v| v).flatten();
            if old.is_none() {
                self.tree
                    .insert(k, bytes)
                    .map_err(|_e| ())
                    .and_then(|_| self.flush())
            } else {
                self.tree
                    .compare_and_swap(k, Some(old.unwrap()), Some(bytes))
                    .map_err(|_e| ())
                    .and_then(|_| self.flush())
            }
        })
    }

    pub fn delete<T: Serialize + DeserializeOwned>(&self, k: &Vec<u8>) -> Result<T, ()> {
        let result = self.read::<T>(k);
        if result.is_some() {
            self.tree
                .remove(k)
                .map_err(|_e| ())
                .and_then(|_| self.flush())?;
            Ok(result.unwrap())
        } else {
            Err(())
        }
    }

    fn flush(&self) -> Result<(), ()> {
        async_std::task::spawn(self.tree.flush_async());
        Ok(())
    }
}

#[test]
fn test_local_file() {
    let name = "test.file";
    let data = "A".to_owned();
    async_std::task::block_on(async {
        write_local_file(name, data.as_bytes()).await.unwrap();
        assert_eq!(
            read_local_file(name).await.ok(),
            Some(data.as_bytes().to_vec())
        );
        assert_eq!(read_string_local_file(name).await.ok(), Some(data));
        remove_local_file(name).await.unwrap();
    });
}

#[test]
fn test_absolute_file() {
    let name = "test.file";
    let data = "A".to_owned();
    let mut path = PathBuf::from("../");
    path.push(name);
    async_std::task::block_on(async {
        write_absolute_file(&path, data.as_bytes()).await.unwrap();
        assert_eq!(
            read_absolute_file(&path).await.ok(),
            Some(data.as_bytes().to_vec())
        );
        assert_eq!(read_string_absolute_file(&path).await.ok(), Some(data));
        remove_absolute_file(&path).await.unwrap();
    });
}

#[test]
fn test_db_file() {
    let name = "test_db";
    let key = vec![1, 2, 3, 4];
    let value = "A".to_owned();
    let value_b = "B".to_owned();

    async_std::task::block_on(async {
        let db = LocalDB::open(name).unwrap();
        assert_eq!(db.write(key.clone(), &value), Ok(()));
        assert_eq!(db.read::<String>(&key), Some(value.clone()));
        assert_eq!(db.update::<String>(key.clone(), &value_b), Ok(()));
        assert_eq!(db.read::<String>(&key), Some(value_b.clone()));
        assert_eq!(db.delete::<String>(&key).ok(), Some(value_b));
        assert_eq!(db.read::<String>(&key), None);
    });
}
