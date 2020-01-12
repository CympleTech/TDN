use async_std::io::Result;
use async_std::sync::{Arc, Receiver, Sender};
use async_std::task;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;

use crate::primitive::RpcParam;
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

    let _send = send; // TODO global in tide

    app.at("/").post(|mut req: tide::Request<()>| {
        async move {
            let body: String = req.body_string().await.unwrap();
            let res: RpcParam = match parse_jsonrpc(body) {
                Ok(_rpc_param) => {
                    println!("TODO");
                    let _res = "";
                    Default::default()
                }
                Err(err) => err.json(),
            };

            tide::Response::new(200)
                .set_mime(mime::APPLICATION_JSON)
                .body_string(res.to_string())
        }
    });

    task::spawn(app.listen(config.addr));

    Ok(())
}

#[derive(Debug, Clone)]
pub enum RpcError {
    ParseError,
    MethodNotFound(u64, String),
    InvalidRequest(u64),
    InvalidVersion(u64),
    InvalidResponse(u64),
}

impl RpcError {
    pub fn json(&self) -> RpcParam {
        match self {
            RpcError::ParseError => json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32700,
                    "message": "Parse error"
                }
            }),
            RpcError::MethodNotFound(id, method) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Method {} not found", method)
                }
            }),
            RpcError::InvalidRequest(id) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32600,
                    "message": "Invalid Request"
                }
            }),
            RpcError::InvalidVersion(id) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32600,
                    "message": "Unsupported JSON-RPC protocol version"
                }
            }),
            RpcError::InvalidResponse(id) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32600,
                    "message": "Invalid Response"
                }
            }),
        }
    }
}

fn parse_jsonrpc(json_string: String) -> std::result::Result<RpcParam, RpcError> {
    match serde_json::from_str::<RpcParam>(&json_string) {
        Ok(value) => {
            let id_res = value
                .get("id")
                .map(|id| {
                    id.as_u64()
                        .or(id.as_str().map(|sid| sid.parse::<u64>().ok()).flatten())
                })
                .flatten();

            if id_res.is_none() {
                return Err(RpcError::ParseError);
            }
            let id = id_res.unwrap();

            // check if json is response
            if value.get("result").is_some() || value.get("error").is_some() {
                return Err(RpcError::InvalidResponse(id));
            }

            if value.get("method").is_none() {
                return Err(RpcError::InvalidRequest(id));
            }

            let jsonrpc = value
                .get("jsonrpc")
                .map(|v| {
                    v.as_str()
                        .map(|s| if s == "2.0" { Some(2) } else { None })
                        .flatten()
                })
                .flatten();

            if jsonrpc.is_none() {
                return Err(RpcError::InvalidVersion(id));
            }

            Ok(value)
        }
        Err(_e) => Err(RpcError::ParseError),
    }
}
