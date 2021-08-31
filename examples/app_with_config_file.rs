use std::path::PathBuf;
use tdn::prelude::*;

#[tokio::main]
async fn main() {
    // use crate root directory's config.toml
    let dir_path = PathBuf::from(".");
    let config = Config::load_with_path(dir_path).await;

    let (peer_addr, _send, mut out_recv) = start_with_config(config).await.unwrap();
    println!("Example: peer id: {}", peer_addr.short_show());

    while let Some(message) = out_recv.recv().await {
        match message {
            _ => {}
        }
    }
}
