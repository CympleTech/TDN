use async_std::task;
use tdn::{new_channel, start};
use tdn_permission::PermissionlessGroup;

fn main() {
    task::block_on(async {
        let (out_send, out_recv) = new_channel();
        let send = start(out_send, PermissionlessGroup::default())
            .await
            .unwrap();

        while let Some(message) = out_recv.recv().await {
            println!("recv: {:?}", message);
            send.send(message).await;
        }
    });
}
