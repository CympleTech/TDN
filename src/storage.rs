use async_std::fs;
use async_std::io::Result;
use std::path::PathBuf;

use crate::primitive::DEFAULT_STORAGE_DIR;

// struct LocalDB {}
// impl LocalDB {
//     async fn read<T>() -> Result<T> {}
//     async fn write<T>() -> Result<()> {}
//     async fn swap<T>() -> Result<()> {}
//     async fn delete<T>() -> Result<T> {}
// }

pub async fn read_local_file(name: &str) -> Result<String> {
    let mut path = DEFAULT_STORAGE_DIR.clone();
    path.push(name);
    fs::read_to_string(name).await
}

pub async fn read_absolute_file(mut path: PathBuf, name: &str) -> Result<String> {
    path.push(name);
    fs::read_to_string(path).await
}

pub async fn write_local_file(name: &str, data: &String) -> Result<()> {
    let mut path = DEFAULT_STORAGE_DIR.clone();
    path.push(name);
    fs::write(path, data.as_bytes()).await
}

pub async fn write_absolute_file(mut path: PathBuf, name: &str, data: &String) -> Result<()> {
    path.push(name);
    fs::write(path, data.as_bytes()).await
}

pub async fn remove_local_file(name: &str) -> Result<()> {
    let mut path = DEFAULT_STORAGE_DIR.clone();
    path.push(name);
    fs::remove_file(name).await
}

pub async fn remove_absolute_file(mut path: PathBuf, name: &str) -> Result<()> {
    path.push(name);
    fs::remove_file(path).await
}

#[test]
fn test_local_file() {
    let name = "test.file";
    let data = "A".to_owned();
    async_std::task::block_on(async {
        write_local_file(name, &data).await.unwrap();
        assert_eq!(read_local_file(name).await.unwrap(), data);
        remove_local_file(name).await.unwrap();
    });
}

#[test]
fn test_absolute_file() {
    let name = "test.file";
    let data = "A".to_owned();
    let path = PathBuf::from("../");
    async_std::task::block_on(async {
        write_absolute_file(path.clone(), name, &data)
            .await
            .unwrap();
        assert_eq!(read_absolute_file(path.clone(), name).await.unwrap(), data);
        remove_absolute_file(path, name).await.unwrap();
    });
}
