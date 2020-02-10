use async_std::fs;
use async_std::io::Result;
use std::path::PathBuf;

use crate::primitive::DEFAULT_STORAGE_DIR;

// pub use chamomile local storage.
pub use chamomile::LocalDB;

pub fn open_db(name: &str) -> Result<LocalDB> {
    let mut path = DEFAULT_STORAGE_DIR.clone();
    path.push(name);
    LocalDB::open_absolute(&path)
}

pub async fn read_local_file(name: &str) -> Result<Vec<u8>> {
    let mut path = DEFAULT_STORAGE_DIR.clone();
    path.push(name);
    fs::read(path).await
}

pub async fn read_string_local_file(name: &str) -> Result<String> {
    let mut path = DEFAULT_STORAGE_DIR.clone();
    path.push(name);
    fs::read_to_string(path).await
}

pub async fn write_local_file(name: &str, data: &[u8]) -> Result<()> {
    let mut path = DEFAULT_STORAGE_DIR.clone();
    path.push(name);
    fs::write(path, data).await
}

pub async fn remove_local_file(name: &str) -> Result<()> {
    let mut path = DEFAULT_STORAGE_DIR.clone();
    path.push(name);
    fs::remove_file(path).await
}

pub async fn read_absolute_file(path: &PathBuf) -> Result<Vec<u8>> {
    fs::read(path).await
}

pub async fn read_string_absolute_file(path: &PathBuf) -> Result<String> {
    fs::read_to_string(path).await
}

pub async fn write_absolute_file(path: &PathBuf, data: &[u8]) -> Result<()> {
    fs::write(path, data).await
}

pub async fn remove_absolute_file(path: &PathBuf) -> Result<()> {
    fs::remove_file(path).await
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
        let db = open_db(name).unwrap();
        assert_eq!(db.write(key.clone(), &value).ok(), Some(()));
        assert_eq!(db.read::<String>(&key), Some(value.clone()));
        assert_eq!(db.update::<String>(key.clone(), &value_b).ok(), Some(()));
        assert_eq!(db.read::<String>(&key), Some(value_b.clone()));
        assert_eq!(db.delete::<String>(&key).ok(), Some(value_b));
        assert_eq!(db.read::<String>(&key), None);
    });
}
