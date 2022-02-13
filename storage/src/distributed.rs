use postcard::{from_bytes, to_allocvec};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    io::{Error, ErrorKind, Result},
    path::PathBuf,
};
use tdn_types::primitives::DEFAULT_STORAGE_DIR;
use tdn_types::storage::Storage;

fn new_error(s: &str) -> Error {
    Error::new(ErrorKind::Other, s)
}

pub fn open_db(name: &str) -> Result<LocalDB> {
    let mut path = DEFAULT_STORAGE_DIR.clone();
    path.push(name);
    LocalDB::open_absolute(&path)
}

pub fn open_absolute_db(mut path: PathBuf, name: &str) -> Result<LocalDB> {
    path.push(name);
    LocalDB::open_absolute(&path)
}

pub struct LocalDB {
    tree: sled::Db,
}

impl Storage for LocalDB {
    type Key = Vec<u8>;

    fn read<T: Serialize + DeserializeOwned>(&self, k: &Self::Key) -> Option<T> {
        self.tree
            .get(k)
            .ok()
            .map(|v| v)
            .flatten()
            .map(|v| from_bytes(&v).ok())
            .flatten()
    }

    fn write<T: Serialize + DeserializeOwned>(&self, k: &Self::Key, t: &T) -> Result<()> {
        to_allocvec(t)
            .map_err(|_e| new_error("db serialize error!"))
            .and_then(|bytes| {
                self.tree
                    .insert(k, bytes)
                    .map_err(|_e| new_error("db write failure!"))
                    .and_then(|_| {
                        self.tree.flush()?;
                        Ok(())
                    })
            })
    }

    fn update<T: Serialize + DeserializeOwned>(&self, k: &Vec<u8>, t: &T) -> Result<()> {
        to_allocvec(&t)
            .map_err(|_e| new_error("db serialize error!"))
            .and_then(|bytes| {
                let old = self.tree.get(&k).ok().map(|v| v).flatten();
                if old.is_none() {
                    self.tree
                        .insert(k, bytes)
                        .map_err(|_e| new_error("db write failure!"))
                        .and_then(|_| {
                            self.tree.flush()?;
                            Ok(())
                        })
                } else {
                    self.tree
                        .compare_and_swap(k, Some(old.unwrap()), Some(bytes))
                        .map_err(|_e| new_error("db swap failure!"))
                        .and_then(|_| {
                            self.tree.flush()?;
                            Ok(())
                        })
                }
            })
    }

    fn delete<T: Serialize + DeserializeOwned>(&self, k: &Self::Key) -> Result<T> {
        let result = self.read::<T>(k);
        if result.is_some() {
            self.tree
                .remove(k)
                .map_err(|_e| new_error("db delete error"))
                .and_then(|_| {
                    self.tree.flush()?;
                    Ok(())
                })?;
            Ok(result.unwrap())
        } else {
            Err(new_error("db delete key not found!"))
        }
    }
}

impl LocalDB {
    pub fn open_absolute(path: &PathBuf) -> Result<LocalDB> {
        let tree = sled::open(path).map_err(|_e| new_error("db open failure!"))?;
        Ok(LocalDB { tree })
    }

    fn _flush(&self) -> Result<()> {
        //smol::spawn(self.tree.flush_async());
        //Ok(())
        todo!();
    }
}

#[test]
fn test_db_file() {
    let name = "test_db";
    let key = vec![1, 2, 3, 4];
    let value = "A".to_owned();
    let value_b = "B".to_owned();

    let db = open_db(name).unwrap();
    assert_eq!(db.write(&key, &value).ok(), Some(()));
    assert_eq!(db.read::<String>(&key), Some(value.clone()));
    assert_eq!(db.update::<String>(&key, &value_b).ok(), Some(()));
    assert_eq!(db.read::<String>(&key), Some(value_b.clone()));
    assert_eq!(db.delete::<String>(&key).ok(), Some(value_b));
    assert_eq!(db.read::<String>(&key), None);
}
