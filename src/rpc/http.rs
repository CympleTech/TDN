use async_std::{
    io,
    io::Result,
    net::{TcpListener, TcpStream},
    sync::{Arc, RwLock, Sender},
    task,
};
use async_tungstenite::{accept_async, tungstenite::protocol::Message as WsMessage};
use futures::{select, sink::SinkExt, FutureExt, StreamExt};
use rand::prelude::*;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use crate::storage::read_string_absolute_file;

use super::{parse_jsonrpc, rpc_channel, RpcMessage};

pub(crate) async fn http_listen(
    index: Option<PathBuf>,
    send: Sender<RpcMessage>,
    listener: TcpListener,
) -> Result<()> {
    let homepage = if let Some(path) = index {
        read_string_absolute_file(&path)
            .await
            .unwrap_or("Error Homepage.".to_owned())
    } else {
        "No Homepage.".to_owned()
    };
    let homelink = Arc::new(RwLock::new(homepage));

    while let Ok((stream, addr)) = listener.accept().await {
        task::spawn(http_connection(
            homelink.clone(),
            send.clone(),
            stream,
            addr,
        ));
    }

    Ok(())
}

// let mut app = tide::with_state(state);

// app.at("/").get(|req: tide::Request<State>| async move {
//     let index_body = if req.state().send.1.is_some() {
//         let path = req.state().send.1.clone().unwrap();
//         read_string_absolute_file(&path).await.ok()
//     } else {
//         None
//     };

//     if index_body.is_some() {
//         tide::Response::new(200)
//             .body_string(index_body.unwrap())
//             .set_mime(mime::TEXT_HTML)
//     } else {
//         tide::Response::new(404)
//             .body_string("Not Found Index Page. --- Power By TDN".to_owned())
//     }
// });

// app.at("/").post(|mut req: tide::Request<State>| {
//     async move {
//         let body: String = req.body_string().await.unwrap();
//         let res = tide::Response::new(200);

//         match parse_jsonrpc(body) {
//             Ok((rpc_param, id)) => {
//                 let (s_send, s_recv) = rpc_channel();
//                 let sender = req.state().send.0.clone();
//                 sender
//                     .send(RpcMessage::Request(id, rpc_param, Some(s_send.clone())))
//                     .await;
//                 drop(sender);

//                 // TODO add timeout.
//                 match s_recv.recv().await {
//                     Some(msg) => {
//                         let param = match msg {
//                             RpcMessage::Response(param) => param,
//                             _ => Default::default(),
//                         };
//                         res.body_string(param.to_string())
//                             .set_mime(mime::APPLICATION_JSON)
//                     }
//                     None => res
//                         .body_string(Default::default())
//                         .set_mime(mime::APPLICATION_JSON),
//                 }
//             }
//             Err((err, id)) => res
//                 .body_string(err.json(id).to_string())
//                 .set_mime(mime::APPLICATION_JSON),
//         }
//     }
// });

// task::spawn(app.listen(config.addr));

async fn http_connection(
    homelink: Arc<RwLock<String>>,
    send: Sender<RpcMessage>,
    raw_stream: TcpStream,
    addr: SocketAddr,
) -> Result<()> {
    debug!("processing jsonrpc request.");
    let (mut reader, mut writer) = &mut (&raw_stream, &raw_stream);

    //io::timeout(Duration::from_secs(5), async { () }).await;

    Ok(())
}
