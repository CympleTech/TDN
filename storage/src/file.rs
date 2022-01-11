use std::path::PathBuf;
use tokio::fs;

type Result<T> = std::io::Result<T>;

pub async fn read_local_file(name: &str) -> Result<Vec<u8>> {
    let mut path = PathBuf::from("./");
    path.push(name);
    fs::read(path).await
}

pub async fn read_string_local_file(name: &str) -> Result<String> {
    let mut path = PathBuf::from("./");
    path.push(name);
    fs::read_to_string(path).await
}

pub async fn write_local_file(name: &str, data: &[u8]) -> Result<()> {
    let mut path = PathBuf::from("./");
    path.push(name);
    fs::write(path, data).await
}

pub async fn remove_local_file(name: &str) -> Result<()> {
    let mut path = PathBuf::from("./");
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

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::main]
    #[test]
    async fn test_local_file() {
        let name = "test.file";
        let data = "A".to_owned();

        write_local_file(name, data.as_bytes()).await.unwrap();
        assert_eq!(
            read_local_file(name).await.ok(),
            Some(data.as_bytes().to_vec())
        );
        assert_eq!(read_string_local_file(name).await.ok(), Some(data));
        remove_local_file(name).await.unwrap();
    }

    #[tokio::main]
    #[test]
    async fn test_absolute_file() {
        let name = "test.file";
        let data = "A".to_owned();
        let mut path = PathBuf::from("../");
        path.push(name);

        write_absolute_file(&path, data.as_bytes()).await.unwrap();
        assert_eq!(
            read_absolute_file(&path).await.ok(),
            Some(data.as_bytes().to_vec())
        );
        assert_eq!(read_string_absolute_file(&path).await.ok(), Some(data));
        remove_absolute_file(&path).await.unwrap();
    }
}
