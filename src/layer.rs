use async_std::{
    io::Result,
    net::{TcpListener, TcpStream},
    prelude::*,
    sync::{channel, Receiver, Sender},
    task,
};
use futures::{select, FutureExt};
use std::collections::HashMap;
use std::net::SocketAddr;

use crate::group::GroupId;
use crate::primitive::MAX_MESSAGE_CAPACITY;
use crate::{new_channel, Message};

// if lower is ture, check black_list -> permissionless
// if lower if false, check white_list -> permissioned
pub struct LayerConfig {
    pub addr: SocketAddr,
    pub lower: bool,
    pub upper: Vec<(SocketAddr, GroupId)>,
    pub white_list: Vec<SocketAddr>,
    pub black_list: Vec<SocketAddr>,
    pub white_group_list: Vec<GroupId>,
    pub black_group_list: Vec<GroupId>,
}

impl LayerConfig {
    pub fn is_close(&self) -> bool {
        self.upper.is_empty()
            && self.lower
            && self.white_list.is_empty()
            && self.white_group_list.is_empty()
    }
}

pub(crate) async fn start(config: LayerConfig, send: Sender<Message>) -> Result<Sender<Message>> {
    let (out_send, out_recv) = new_channel();
    let (self_send, self_recv) = channel::<StreamMessage>(MAX_MESSAGE_CAPACITY);

    if config.is_close() {
        return Ok(out_send);
    }

    let listener = TcpListener::bind(config.addr).await?;
    task::spawn(run_listener(listener, send.clone(), self_send.clone()));
    task::spawn(run_receiver(config, out_recv, send, self_send, self_recv));

    Ok(out_send)
}

async fn run_listener(
    listener: TcpListener,
    send: Sender<Message>,
    self_send: Sender<StreamMessage>,
) -> Result<()> {
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        task::spawn(process_stream(
            stream?,
            send.clone(),
            self_send.clone(),
            false,
        ));
    }

    drop(incoming);
    drop(send);
    Ok(())
}

async fn run_client(
    addr: SocketAddr,
    bytes: Vec<u8>,
    send: Sender<Message>,
    self_send: Sender<StreamMessage>,
) -> Result<()> {
    let mut stream = TcpStream::connect(addr).await?;
    let len = bytes.len() as u32;
    stream.write(&(len.to_be_bytes())).await?;
    stream.write_all(&bytes[..]).await?;
    task::spawn(process_stream(stream, send, self_send, true));

    Ok(())
}

async fn run_receiver(
    config: LayerConfig,
    mut out_recv: Receiver<Message>,
    send: Sender<Message>,
    self_send: Sender<StreamMessage>,
    mut self_recv: Receiver<StreamMessage>,
) -> Result<()> {
    let mut uppers: HashMap<GroupId, HashMap<u32, Sender<StreamMessage>>> = HashMap::new();
    let mut lowers: HashMap<GroupId, HashMap<u32, Sender<StreamMessage>>> = HashMap::new();

    // link to uppers
    for (addr, gid) in config.upper {
        uppers.insert(gid, HashMap::new());
        // TODO link and join bytes.
        task::spawn(run_client(addr, vec![], send.clone(), self_send.clone()));
    }

    loop {
        select! {
            msg = out_recv.next().fuse() => match msg {
                Some(msg) => {
                    println!("recv from outside: {:?}", msg);
                    match msg {
                        Message::Upper(gid, data) => {}
                        Message::Lower(gid, data) => {}
                        _ => {}
                    }
                }
                None => break,
            },
            msg = self_recv.next().fuse() => match msg {
                Some(msg) => {
                    match msg {
                        StreamMessage::Open(gid, uid, sender) => {},
                        StreamMessage::Close(gid, uid) => {},
                        _ => {}
                    }
                }
                None => break,
            }
        }
    }

    drop(send);
    drop(self_send);
    drop(out_recv);
    drop(self_recv);

    Ok(())
}

enum StreamMessage {
    Open(GroupId, u32, Sender<StreamMessage>),
    Close(GroupId, u32),
    Data(Vec<u8>),
}

async fn process_stream(
    stream: TcpStream,
    sender: Sender<Message>,
    server_send: Sender<StreamMessage>,
    is_upper: bool,
) -> Result<()> {
    let addr = stream.peer_addr()?;

    let (mut reader, mut writer) = &mut (&stream, &stream);

    let (self_send, self_recv) = channel::<StreamMessage>(MAX_MESSAGE_CAPACITY);
    let uid = rand::random::<u32>();

    // TODO read gid

    let gid = GroupId::default();

    server_send
        .send(StreamMessage::Open(gid, uid, self_send))
        .await;

    let mut read_len = [0u8; 4];

    loop {
        select! {
            msg = reader.read(&mut read_len).fuse() => match msg {
                Ok(size) => {
                    if size == 0 {
                        // when close or better when many Ok(0)
                        server_send.send(StreamMessage::Close(gid, uid)).await;
                        break;
                    }

                    let len: usize = u32::from_be_bytes(read_len) as usize;
                    let mut read_bytes = vec![0u8; len];
                    while let Ok(bytes_size) = reader.read(&mut read_bytes).await {
                        if bytes_size != len {
                            break;
                        }

                        let message = if is_upper {
                            Message::Upper(gid, read_bytes.clone())
                        } else {
                            Message::Lower(gid, read_bytes.clone())
                        };

                        sender.send(message).await;
                        break;
                    }
                    read_len = [0u8; 4];
                }
                Err(e) => {
                    server_send.send(StreamMessage::Close(gid, uid)).await;
                    break;
                }
            },
            msg = self_recv.recv().fuse() => match msg {
                Some(msg) => {
                    match msg {
                        StreamMessage::Data(bytes) => {
                            let len = bytes.len() as u32;
                            writer.write(&(len.to_be_bytes())).await?;
                            writer.write_all(&bytes[..]).await?;
                        }
                        StreamMessage::Close(_, _) => break,
                        _ => break,
                    }
                },
                None => break,
            }
        }
    }

    println!("close stream: {}", addr);

    Ok(())
}
