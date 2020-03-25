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
use std::collections::{hash_map::Entry, HashMap};
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use crate::message::{LayerReceiveMessage, LayerSendMessage, ReceiveMessage};
use crate::primitive::{GroupId, MAX_MESSAGE_CAPACITY};

// if layer open is ture, check black_list -> permissionless
// if layer open if false, check white_list -> permissioned
pub struct LayerConfig {
    pub addr: SocketAddr,
    pub public: bool,
    pub upper: Vec<(SocketAddr, GroupId)>,
    pub white_list: Vec<IpAddr>,
    pub black_list: Vec<IpAddr>,
    pub white_group_list: Vec<GroupId>,
    pub black_group_list: Vec<GroupId>,
}

struct Layer {
    pub public: bool,
    pub white_list: Vec<IpAddr>,
    pub black_list: Vec<IpAddr>,
    pub white_group_list: Vec<GroupId>,
    pub black_group_list: Vec<GroupId>,
    pub upper: Vec<(SocketAddr, GroupId)>,
}

impl Layer {
    pub fn is_close(&self) -> bool {
        self.upper.is_empty()
            && !self.public
            && self.white_list.is_empty()
            && self.white_group_list.is_empty()
    }

    fn black_contains(&self, addr: &SocketAddr, gid: &GroupId) -> bool {
        self.black_list.contains(&addr.ip()) || self.black_group_list.contains(gid)
    }

    fn white_contains(&self, addr: &SocketAddr, gid: &GroupId) -> bool {
        self.white_list.contains(&addr.ip()) || self.white_group_list.contains(gid)
    }

    fn generate_remote_public(&self, r_gid: GroupId, gid: GroupId) -> RemotePublic {
        RemotePublic(r_gid, gid, vec![])
    }
}

/// new a channel, send message to layers Message. default capacity is 1024.
fn new_send_channel() -> (Sender<LayerSendMessage>, Receiver<LayerSendMessage>) {
    channel(MAX_MESSAGE_CAPACITY)
}

pub(crate) async fn start(
    gid: GroupId,
    config: LayerConfig,
    send: Sender<ReceiveMessage>,
) -> Result<Sender<LayerSendMessage>> {
    let (out_send, out_recv) = new_send_channel();
    let (self_send, self_recv) = channel::<StreamMessage>(MAX_MESSAGE_CAPACITY);
    let (addr, default_layer) = (
        config.addr,
        Layer {
            public: config.public,
            white_list: config.white_list,
            black_list: config.black_list,
            white_group_list: config.white_group_list,
            black_group_list: config.black_group_list,
            upper: config.upper,
        },
    );

    if default_layer.is_close() {
        return Ok(out_send);
    }

    let listener = TcpListener::bind(addr).await?;
    task::spawn(run_listener(listener, send.clone(), self_send.clone()));
    task::spawn(run_receiver(
        gid,
        default_layer,
        out_recv,
        send,
        self_send,
        self_recv,
    ));

    Ok(out_send)
}

async fn run_listener(
    listener: TcpListener,
    send: Sender<ReceiveMessage>,
    self_send: Sender<StreamMessage>,
) -> Result<()> {
    let mut incoming = listener.incoming();
    while let Some(Ok(stream)) = incoming.next().await {
        task::spawn(process_stream(
            None,
            stream,
            send.clone(),
            self_send.clone(),
        ));
    }

    drop(incoming);
    drop(send);
    Ok(())
}

async fn run_client(
    remote_public: RemotePublic,
    addr: SocketAddr,
    send: Sender<ReceiveMessage>,
    self_send: Sender<StreamMessage>,
) -> Result<()> {
    let stream = TcpStream::connect(addr).await?;
    task::spawn(process_stream(Some(remote_public), stream, send, self_send));

    Ok(())
}

type LayerBuffer = HashMap<GroupId, HashMap<u32, Sender<StreamMessage>>>;

fn layer_buffer_insert(map: &mut LayerBuffer, gid: GroupId, uid: u32, send: Sender<StreamMessage>) {
    match map.entry(gid) {
        Entry::Occupied(mut entry) => {
            entry.get_mut().insert(uid, send);
        }
        Entry::Vacant(entry) => {
            let mut h = HashMap::new();
            h.insert(uid, send);
            entry.insert(h);
        }
    }
}

async fn run_receiver(
    default_gid: GroupId,
    default_layer: Layer,
    out_recv: Receiver<LayerSendMessage>,
    send: Sender<ReceiveMessage>,
    self_send: Sender<StreamMessage>,
    self_recv: Receiver<StreamMessage>,
) -> Result<()> {
    let mut layers: HashMap<GroupId, Layer> = HashMap::new();
    layers.insert(default_gid, default_layer);

    let mut uppers: LayerBuffer = HashMap::new();
    let mut tmp_uppers: LayerBuffer = HashMap::new();
    let mut lowers: LayerBuffer = HashMap::new();
    let mut tmp_lowers: LayerBuffer = HashMap::new();

    for (gid, layer) in layers.iter_mut() {
        // link to uppers
        for (addr, r_gid) in &layer.upper {
            let remote_public = layer.generate_remote_public(*r_gid, *gid);
            layer.white_list.push(addr.ip());
            task::spawn(run_client(
                remote_public.clone(),
                *addr,
                send.clone(),
                self_send.clone(),
            ));
        }
    }

    loop {
        select! {
            msg = out_recv.recv().fuse() => match msg {
                Some(msg) => {
                    println!("DEBUG: recv from outside: {:?}", msg);
                    match msg {
                        LayerSendMessage::Upper(gid, data) => {
                            uppers.get(&gid).map(|h| {
                                let _ = h.iter().map(|(_u, sender)| {
                                    let data = data.clone();
                                    async move {
                                        sender.send(StreamMessage::Data(data)).await;
                                    }
                                });
                            });
                        }
                        LayerSendMessage::Lower(gid, data) => {
                            lowers.get(&gid).map(|h| {
                                let _ = h.iter().map(|(_u, sender)| {
                                    let data = data.clone();
                                    async move {
                                        sender.send(StreamMessage::Data(data)).await;
                                    }
                                });
                            });
                        }
                        LayerSendMessage::UpperJoin(gid) => {
                            // TODO start a upper to listener.
                            if layers.contains_key(&gid) {
                                continue;
                            }
                            // let layer = Layer {
                            //     public: config.public,
                            //     white_list: config.white_list,
                            //     black_list: config.black_list,
                            //     white_group_list: config.white_group_list,
                            //     black_group_list: config.black_group_list,
                            //     upper: config.upper,
                            // }
                        }
                        LayerSendMessage::UpperLeave(gid) => {
                            // TODO remove a upper to listener.
                        }
                        LayerSendMessage::LowerJoin(gid, remote_gid, _uid, addr, join_data) => {
                            let _ = layers.get_mut(&gid).map(|layer| {
                                // handle to upper
                                layer.white_list.push(addr.ip());
                                let remote_public = layer.generate_remote_public(remote_gid, gid);

                                // TODO if join data
                                if !uppers.contains_key(&gid) {
                                    task::spawn(run_client(
                                        remote_public.clone(),
                                        addr,
                                        send.clone(),
                                        self_send.clone(),
                                    ));
                                }
                            });
                        }
                        LayerSendMessage::LowerJoinResult(gid, remote_gid, uid, is_ok) => {
                            // handle to lowers
                            match tmp_lowers.get_mut(&gid) {
                                Some(h) => {
                                    match h.remove(&uid) {
                                        Some(sender) => if !is_ok {
                                            sender.send(
                                                StreamMessage::Close(gid, uid, true)
                                            ).await;
                                        } else {
                                            let layer = layers.get(&gid).unwrap();
                                            sender.send(StreamMessage::Ok(layer.generate_remote_public(remote_gid, gid))).await;
                                            layer_buffer_insert(&mut lowers, gid, uid, sender);
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                },
                None => break,
            },
            msg = self_recv.recv().fuse() => match msg {
                Some(msg) => {
                    match msg {
                        StreamMessage::Open(gid, remote_gid, uid, addr, sender, is_upper) => {
                            println!("DEBUG: layer: {}, uid: {} open ok!", remote_gid.short_show(), uid);
                            if !layers.contains_key(&gid) {
                                continue;
                            }
                            let layer = layers.get(&gid).unwrap();

                            if layer.black_contains(&addr, &remote_gid) {
                                continue;
                            }

                            let is_white = layer.white_contains(&addr, &remote_gid);

                            let entry = if is_upper {
                                if is_white {
                                    // remove tmp_uppers
                                    tmp_uppers.get_mut(&gid).map(|h| h.remove(&uid));
                                    send.send(ReceiveMessage::Layer(
                                        LayerReceiveMessage::LowerJoinResult(
                                            gid, remote_gid, uid, true
                                        ))).await;

                                    &mut uppers
                                } else {
                                    continue;
                                }
                            } else {
                                if is_white {
                                    sender.send(StreamMessage::Ok(layer.generate_remote_public(remote_gid, gid))).await;

                                    &mut lowers
                                } else {
                                    if layer.public {
                                        send.send(
                                            ReceiveMessage::Layer(
                                                LayerReceiveMessage::LowerJoin(
                                                    gid, remote_gid, uid, addr, vec![]) // TODO
                                            )).await;

                                        &mut tmp_lowers
                                    } else {
                                        continue;
                                    }
                                }
                            };

                            layer_buffer_insert(entry, remote_gid, uid, sender);
                        },
                        StreamMessage::Close(gid, uid, is_upper) => {
                            let (entry, tmp_entry) = if is_upper {
                                (&mut uppers, &mut tmp_uppers)
                            } else {
                                (&mut lowers, &mut tmp_lowers)
                            };

                            tmp_entry.get_mut(&gid).map(|h| {
                                h.remove(&uid);
                            });

                            entry.get_mut(&gid).map(|h| {
                                h.remove(&uid);
                                // TODO new link to this entry
                            });
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
    has_remote_public: Option<RemotePublic>,
    stream: TcpStream,
    sender: Sender<ReceiveMessage>,
    server_send: Sender<StreamMessage>,
) -> Result<()> {
    let is_upper = has_remote_public.is_some();
    println!("DEBUG: start process stream");
    let addr = stream.peer_addr()?;
    let (mut reader, mut writer) = &mut (&stream, &stream);

    // if is to upper, send self-info first.
    if is_upper {
        let remote_public_bytes = has_remote_public.unwrap().to_bytes();
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

    let RemotePublic(gid, r_gid, _bytes) = result.unwrap();

    // TODO Security DH exchange.

    let (self_send, mut self_recv) = channel::<StreamMessage>(MAX_MESSAGE_CAPACITY);
    let uid = rand::random::<u32>();

    server_send
        .send(StreamMessage::Open(
            gid, r_gid, uid, addr, self_send, is_upper,
        ))
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
                            ReceiveMessage::Layer(LayerReceiveMessage::Upper(gid, read_bytes.clone()))
                        } else {
                            ReceiveMessage::Layer(LayerReceiveMessage::Lower(gid, read_bytes.clone()))
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
                        StreamMessage::Ok(remote_public) => {
                            println!("DEBUG: send remote after verify");
                            let remote_public_bytes = remote_public.to_bytes();
                            let len = remote_public_bytes.len() as u32;
                            writer.write(&(len.to_be_bytes())).await?;
                            writer.write_all(&remote_public_bytes[..]).await?;
                        }
                        _ => break,
                    }
                },
                None => break,
            }
        }
    }

    println!("DEBUG: close layers: {}", addr);
    server_send
        .send(StreamMessage::Close(gid, uid, is_upper))
        .await;

    Ok(())
}

#[derive(Debug)]
enum StreamMessage {
    Open(
        GroupId, // request group
        GroupId, // self group
        u32,
        SocketAddr,
        Sender<StreamMessage>,
        bool, // is_upper
    ),
    Close(GroupId, u32, bool),
    Data(Vec<u8>),
    Ok(RemotePublic),
}

// Rtemote Public Info, include local transport and public key bytes.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RemotePublic(pub GroupId, pub GroupId, pub Vec<u8>);

impl RemotePublic {
    pub fn from_bytes(bytes: Vec<u8>) -> std::result::Result<Self, ()> {
        bincode::deserialize(&bytes).map_err(|_e| ())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}
