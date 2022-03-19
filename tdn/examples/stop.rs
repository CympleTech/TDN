use std::{path::PathBuf, time::Duration};
use tdn::prelude::*;

#[tokio::main]
async fn main() {
    let config = Config::load(PathBuf::from("./")).await;
    let (_secret, ids, p2p_config, _rpc_config) = config.split();
    let (send_send, send_recv) = new_send_channel();
    let (recv_send, mut recv_recv) = new_receive_channel();

    let peer_id = start_main(ids, p2p_config, recv_send, send_recv, None, None)
        .await
        .unwrap();
    println!("Example: peer id: {:?}", peer_id);

    println!("Network will stop after 5s...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    let _ = send_send
        .send(SendMessage::Network(NetworkType::NetworkStop))
        .await;

    while let Some(_) = recv_recv.recv().await {
        //
    }

    println!("Network is stopped.");
}
