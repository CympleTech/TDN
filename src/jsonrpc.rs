use async_std::io::Result;
use async_std::sync::{Arc, Receiver, Sender};
use async_std::task;
use jsonrpc_core::IoHandler;
use std::net::SocketAddr;

use crate::primitive::{RpcParam, RpcValue};
use crate::{new_channel, Message};

pub struct RpcConfig {
    pub addr: SocketAddr,
}

pub(crate) async fn start(config: RpcConfig, send: Sender<Message>) -> Result<Sender<Message>> {
    let (out_send, out_recv) = new_channel();

    // start json rpc server

    // start listen self recv

    server(send.clone(), config).await?;
    listen(send, out_recv).await?;

    Ok(out_send)
}

async fn listen(send: Sender<Message>, out_recv: Receiver<Message>) -> Result<()> {
    task::spawn(async move {
        while let Some(_msg) = out_recv.recv().await {
            println!("msg");
        }

        drop(send);
    });
    Ok(())
}

async fn server(send: Sender<Message>, config: RpcConfig) -> Result<()> {
    let mut app = tide::new();

    let mut io: IoHandler = IoHandler::default();
    io.add_method("hello", |_params: RpcParam| {
        Ok(RpcValue::String("hello".into()))
    });

    let _send = send; // global in tide
    let _io = Arc::new(io); // TODO global with tide

    app.at("/").post(|mut req: tide::Request<()>| {
        let mut io: IoHandler = IoHandler::default();
        io.add_method("hello", |params: RpcParam| {
            println!("{:?}", params);
            Ok(RpcValue::String("hello".into()))
        });

        async move {
            let body: String = req.body_string().await.unwrap();
            tide::Response::new(200)
                .set_mime(mime::APPLICATION_JSON)
                .body_string(
                io.handle_request_sync(&body).unwrap_or(
                    r#"{"error": {"code": -32600, "message": "Invalid JSONRPC Request"}, "jsonrpc": "2.0"}"#
                        .to_owned(),
                ),
            )
        }
    });

    task::spawn(app.listen(config.addr));

    Ok(())
}
