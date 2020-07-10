use async_std::{
    io::Result,
    net::{TcpListener, TcpStream},
    prelude::*,
    sync::{Arc, RwLock, Sender},
    task,
};
use rand::prelude::*;
use std::net::SocketAddr;
use std::path::PathBuf;
//use std::time::Duration;

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

enum HTTP {
    Ok,
    NeedMore(usize),
}

fn parse_req<'a>(src: Vec<u8>, had_body: &mut Vec<u8>) -> std::result::Result<HTTP, &'a str> {
    if had_body.len() == 0 {
        let mut req_parsed_headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut req_parsed_headers);
        let status = req.parse(&src).map_err(|_| "HTTP parse error")?;

        let content_length_headers: Vec<httparse::Header> = req
            .headers
            .iter()
            .filter(|header| header.name == "Content-Length")
            .cloned()
            .collect();

        if content_length_headers.len() != 1 {
            return Err("HTTP header is invalid");
        }

        let length_bytes = content_length_headers.first().unwrap().value;
        let mut length_string = String::new();

        for b in length_bytes {
            length_string.push(*b as char);
        }

        let length = length_string
            .parse::<usize>()
            .map_err(|_| "HTTP length is invalid")?;

        let amt = match status {
            httparse::Status::Complete(amt) => amt,
            httparse::Status::Partial => return Err("HTTP parse error"),
        };

        let (_pre, had) = src.split_at(amt);
        had_body.extend(had);

        if had_body.len() < length {
            return Ok(HTTP::NeedMore(length));
        }
    }

    Ok(HTTP::Ok)
}

async fn http_connection(
    _homelink: Arc<RwLock<String>>,
    send: Sender<RpcMessage>,
    mut stream: TcpStream,
    addr: SocketAddr,
) -> Result<()> {
    debug!("DEBUG: HTTP connection established: {}", addr);
    let id: u64 = rand::thread_rng().gen();
    let (s_send, s_recv) = rpc_channel();

    let mut buf = vec![];
    let mut length = 0;

    // TODO add timeout
    loop {
        let mut tmp_buf = vec![0u8; 1024];
        let n = stream.read(&mut tmp_buf).await?;
        let tmp_buf = tmp_buf[..n].to_vec();

        if n == 0 {
            break;
        }

        if buf.len() == 0 {
            match parse_req(tmp_buf, &mut buf) {
                Ok(HTTP::NeedMore(len)) => {
                    length = len;
                }
                Ok(HTTP::Ok) => break,
                Err(e) => {
                    info!("TDN: HTTP JSONRPC parse error: {}", e);
                    return Ok(());
                }
            }
        } else {
            buf.extend(&tmp_buf[0..n]);
            if buf.len() >= length {
                break;
            }
        }
    }

    let msg = String::from_utf8_lossy(&buf[..]);
    let res = "HTTP/1.1 200 OK\r\n\r\n";

    match parse_jsonrpc((*msg).to_string()) {
        Ok((rpc_param, _id)) => {
            send.send(RpcMessage::Request(id, rpc_param, Some(s_send)))
                .await;
        }
        Err((err, id)) => {
            stream
                .write(format!("{}{}", res, err.json(id).to_string()).as_bytes())
                .await?;
            stream.flush();
            stream.shutdown(std::net::Shutdown::Both)?;
        }
    }

    while let Ok(msg) = s_recv.recv().await {
        let param = match msg {
            RpcMessage::Response(param) => param,
            _ => Default::default(),
        };
        stream
            .write(format!("{}{}", res, param.to_string()).as_bytes())
            .await?;
        stream.flush();
        stream.shutdown(std::net::Shutdown::Both)?;
        break;
    }

    Ok(())
}
