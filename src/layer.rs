use async_std::{
    io,
    io::Result,
    net::{TcpListener, TcpStream},
    prelude::*,
    sync::{channel, Receiver, Sender},
    task,
};
use futures::{select, FutureExt};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

use crate::primitive::{GroupId, MAX_MESSAGE_CAPACITY};
use crate::{new_channel, LayerMessage, Message};

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

pub(crate) async fn start(
    gid: GroupId,
    config: LayerConfig,
    send: Sender<Message>,
) -> Result<Sender<Message>> {
    let (out_send, out_recv) = new_channel();
    let (self_send, self_recv) = channel::<StreamMessage>(MAX_MESSAGE_CAPACITY);

    if config.is_close() {
        return Ok(out_send);
    }

    let remote_public = RemotePublic(gid, vec![]);

    let listener = TcpListener::bind(config.addr).await?;
    task::spawn(run_listener(
        remote_public.clone(),
        listener,
        send.clone(),
        self_send.clone(),
    ));
    task::spawn(run_receiver(
        remote_public,
        config,
        out_recv,
        send,
        self_send,
        self_recv,
    ));

    Ok(out_send)
}

async fn run_listener(
    remote_public: RemotePublic,
    listener: TcpListener,
    send: Sender<Message>,
    self_send: Sender<StreamMessage>,
) -> Result<()> {
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        task::spawn(process_stream(
            remote_public.clone(),
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
    remote_public: RemotePublic,
    addr: SocketAddr,
    send: Sender<Message>,
    self_send: Sender<StreamMessage>,
) -> Result<()> {
    let stream = TcpStream::connect(addr).await?;
    task::spawn(process_stream(
        remote_public.clone(),
        stream,
        send,
        self_send,
        true,
    ));

    Ok(())
}

async fn run_receiver(
    remote_public: RemotePublic,
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
        task::spawn(run_client(
            remote_public.clone(),
            addr,
            send.clone(),
            self_send.clone(),
        ));
    }

    loop {
        select! {
            msg = out_recv.next().fuse() => match msg {
                Some(msg) => {
                    println!("recv from outside: {:?}", msg);
                    match msg {
                        Message::Layer(LayerMessage::Upper(gid, data)) => {

                        }
                        Message::Layer(LayerMessage::Lower(gid, data)) => {}
                        _ => {}
                    }
                }
                None => break,
            },
            msg = self_recv.next().fuse() => match msg {
                Some(msg) => {
                    match msg {
                        StreamMessage::Open(gid, uid, sender, is_upper) => {
                            let entry = if is_upper {
                                &mut uppers
                            } else {
                                &mut lowers
                            };

                            entry.entry(gid).and_modify(|h| {
                                h.insert(uid, sender.clone());
                            }).or_insert({
                                let mut h = HashMap::new();
                                h.insert(uid, sender);
                                h
                            });

                            println!("layer: {}, uid: {} open ok!", gid.short_show(), uid);
                        },
                        StreamMessage::Close(gid, uid, is_upper) => {
                            let entry = if is_upper {
                                &mut uppers
                            } else {
                                &mut lowers
                            };

                            entry.get_mut(&gid).map(|h| {
                                h.remove(&uid);
                                // TODO new link to this entry
                            });

                            println!("layer: {}, uid: {} closed!", gid.short_show(), uid);
                        },
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

async fn process_stream(
    remote_public: RemotePublic,
    stream: TcpStream,
    sender: Sender<Message>,
    server_send: Sender<StreamMessage>,
    is_upper: bool,
) -> Result<()> {
    println!("DEBUG: start process stream");
    let addr = stream.peer_addr()?;
    let (mut reader, mut writer) = &mut (&stream, &stream);

    let remote_public_bytes = remote_public.to_bytes();
    if is_upper {
        println!("DEBUG: send remote by self");
        let len = remote_public_bytes.len() as u32;
        writer.write(&(len.to_be_bytes())).await?;
        writer.write_all(&remote_public_bytes[..]).await?;
    }

    // timeout 10s to read peer_id & public_key
    let result: Result<Option<RemotePublic>> = io::timeout(Duration::from_secs(5), async {
        let mut read_len = [0u8; 4];
        while let Ok(size) = reader.read(&mut read_len).await {
            if size == 0 {
                // when close or better when many Ok(0)
                break;
            }

            let len: usize = u32::from_be_bytes(read_len) as usize;
            let mut read_bytes = vec![0u8; len];
            while let Ok(bytes_size) = reader.read(&mut read_bytes).await {
                if bytes_size != len {
                    break;
                }

                return Ok(RemotePublic::from_bytes(read_bytes).ok());
            }
        }
        Ok(None)
    })
    .await;

    if result.is_err() {
        println!("Debug: Session timeout");
        return Ok(());
    }

    let result = result.unwrap();
    if result.is_none() {
        println!("Debug: Session invalid pk");
        return Ok(());
    }

    let RemotePublic(gid, _bytes) = result.unwrap();

    // TODO verify upper/lower.

    // if verify ok, send self public info.
    if !is_upper {
        println!("DEBUG: send remote after verify");
        let len = remote_public_bytes.len() as u32;
        writer.write(&(len.to_be_bytes())).await?;
        writer.write_all(&remote_public_bytes[..]).await?;
    }

    // TODO Security DH exchange.

    let (self_send, mut self_recv) = channel::<StreamMessage>(MAX_MESSAGE_CAPACITY);
    let uid = rand::random::<u32>();

    server_send
        .send(StreamMessage::Open(gid, uid, self_send, is_upper))
        .await;

    let mut read_len = [0u8; 4];

    loop {
        select! {
            msg = reader.read(&mut read_len).fuse() => match msg {
                Ok(size) => {
                    if size == 0 {
                        break;
                    }

                    let len: usize = u32::from_be_bytes(read_len) as usize;
                    let mut read_bytes = vec![0u8; len];
                    while let Ok(bytes_size) = reader.read(&mut read_bytes).await {
                        if bytes_size != len {
                            break;
                        }

                        let message = if is_upper {
                            Message::Layer(LayerMessage::Upper(gid, read_bytes.clone()))
                        } else {
                            Message::Layer(LayerMessage::Lower(gid, read_bytes.clone()))
                        };
                        sender.send(message).await;
                        break;
                    }
                    read_len = [0u8; 4];
                }
                Err(_e) => break,
            },
            msg = self_recv.next().fuse() => match msg {
                Some(msg) => {
                    match msg {
                        StreamMessage::Data(bytes) => {
                            let len = bytes.len() as u32;
                            writer.write(&(len.to_be_bytes())).await?;
                            writer.write_all(&bytes[..]).await?;
                        }
                        _ => break,
                    }
                },
                None => break,
            }
        }
    }

    println!("close layers: {}", addr);
    server_send
        .send(StreamMessage::Close(gid, uid, is_upper))
        .await;

    Ok(())
}

#[derive(Debug)]
enum StreamMessage {
    Open(GroupId, u32, Sender<StreamMessage>, bool),
    Close(GroupId, u32, bool),
    Data(Vec<u8>),
}

// Rtemote Public Info, include local transport and public key bytes.
#[derive(Deserialize, Serialize, Clone)]
pub struct RemotePublic(pub GroupId, pub Vec<u8>);

impl RemotePublic {
    pub fn from_bytes(bytes: Vec<u8>) -> std::result::Result<Self, ()> {
        bincode::deserialize(&bytes).map_err(|_e| ())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}
