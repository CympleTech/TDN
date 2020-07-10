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
    Ok(usize),
    NeedMore(usize, usize),
}

fn parse_req<'a>(src: &[u8]) -> std::result::Result<HTTP, &'a str> {
    let mut req_parsed_headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut req_parsed_headers);
    let status = req.parse(&src).map_err(|_| "HTTP parse error")?;

    let content_length_headers: Vec<httparse::Header> = req
        .headers
        .iter()
        .filter(|header| header.name.to_ascii_lowercase() == "content-length")
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

    if src[amt..].len() >= length {
        return Ok(HTTP::Ok(amt));
    }

    Ok(HTTP::NeedMore(amt, length))
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

    // TODO add timeout
    let mut tmp_buf = vec![0u8; 1024];
    let n = stream.read(&mut tmp_buf).await?;
    let body = match parse_req(&tmp_buf[..n]) {
        Ok(HTTP::NeedMore(amt, len)) => {
            buf.extend(&tmp_buf[amt..n]);
            loop {
                let mut tmp = vec![0u8; 1024];
                let n = stream.read(&mut tmp).await?;
                buf.extend(&tmp[..n]);
                if buf.len() >= len {
                    break;
                }
            }
            &buf[..]
        }
        Ok(HTTP::Ok(amt)) => &tmp_buf[amt..n],
        Err(e) => {
            info!("TDN: HTTP JSONRPC parse error: {}", e);
            return Ok(());
        }
    };

    let msg = String::from_utf8_lossy(body);
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
