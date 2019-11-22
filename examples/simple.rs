use async_std::task;
use async_std::sync::channel;
use tdn::{new_channel, start};

fn main() {
    task::block_on(async {
        let (out_send, out_recv) = new_channel();
        let send = start(out_send).await.unwrap();

        while let Some(message) = out_recv.recv().await {
            println!("recv: {:?}", message);
            send.send(message).await;
        }
    });
}
