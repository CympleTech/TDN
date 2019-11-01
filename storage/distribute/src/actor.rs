use serde_json::json;
use std::collections::{HashMap, HashSet};

use rckad::KadTree;
use teatree::actor::prelude::*;
use teatree::crypto::keypair::{PrivateKey, PublicKey, PUBLIC_KEY_LENGTH};
use teatree::primitives::functions::get_default_storage_path;
use teatree::primitives::types::{GroupID, PeerAddr};
use teatree::storage::{DiskStorageActor, EntityRead, EntityWrite};
use teatree::traits::actor::BridgeActor;
use teatree::traits::message::bridge_message::*;
use teatree::Configure;
use teatree::NetworkBridgeActor;

use crate::event::{Event, EventType};
use crate::message::{Drop, Read, ReadResult, Write, WriteResult, ID};
use crate::primitives::DEFAULT_STORAGE_NAME;
use crate::read::{AppendTaskMessage, ReadActor, ReadCloseMessage, ReadResultMessage};
use crate::store::DataStore;

pub struct DSActor {
    group_id: GroupID,
    pk: PublicKey,
    psk: PrivateKey,
    bridge: Addr<NetworkBridgeActor>,
    storage: Addr<DiskStorageActor>,
    dht: KadTree<[u8; PUBLIC_KEY_LENGTH], PeerAddr>,
    stores: HashSet<ID>,
    read_tasks: HashMap<ID, Addr<ReadActor>>,
}

impl DSActor {
    pub fn load(group_id: GroupID, bridge: Addr<NetworkBridgeActor>) -> Self {
        let psk = PrivateKey::generate();
        Self::load_with_psk(group_id, bridge, psk)
    }

    pub fn load_with_psk(
        group_id: GroupID,
        bridge: Addr<NetworkBridgeActor>,
        psk: PrivateKey,
    ) -> Self {
        let mut path = get_default_storage_path();
        path.push(DEFAULT_STORAGE_NAME);
        path.push(format!("{}", group_id));

        let pk = psk.generate_public_key();
        println!("DEBUG: DS PK: {}", pk);

        let storage = DiskStorageActor::run(Some(path));
        let dht = KadTree::new(pk.to_bytes(), pk.clone());
        let stores = HashSet::new();
        let read_tasks = HashMap::new();

        Self {
            group_id,
            pk,
            psk,
            bridge,
            storage,
            dht,
            stores,
            read_tasks,
        }
    }

    fn handle_event_type(&mut self, event_type: EventType, ctx: &mut <Self as Actor>::Context) {
        match event_type {
            EventType::Read(pk, id) => {
                println!("DEBUG: Read Event, ID: {}", id);
                if self.stores.contains(&id) {
                    self.storage
                        .send(EntityRead::<DataStore>(id.clone()))
                        .into_actor(self)
                        .then(move |res, act, _ctx| {
                            match res {
                                Ok(Ok(e)) => {
                                    if let Some((_, peer_addr, _)) = act.dht.search(&pk.as_bytes())
                                    {
                                        act.bridge.do_send(EventMessage(
                                            act.group_id.clone(),
                                            peer_addr.clone(),
                                            bincode::serialize(&Event::new_read_result(
                                                pk, id, e.1, &act.pk, &act.psk,
                                            ))
                                            .unwrap_or(vec![]),
                                        ))
                                    }
                                }
                                _ => {}
                            };
                            actor_ok(())
                        })
                        .wait(ctx);
                }
            }
            EventType::Write(_pk, id, data) => {
                println!("DEBUG: Write Event, ID: {}", id);
                self.storage
                    .do_send(EntityWrite(DataStore(id.clone(), data)));
                self.stores.insert(id);
            }
            EventType::ReadResult(_pk, id, data) => {
                println!("DEBUG: Read Result Event, ID: {}", id);
                self.read_tasks.get(&id).map(|addr| {
                    addr.do_send(ReadResultMessage(data));
                });
            }
            _ => {}
        }
    }

    fn read(
        &mut self,
        id: ID,
        index: usize,
        addr: Option<Recipient<ReadResult>>,
        ctx: &mut <Self as Actor>::Context,
    ) {
        if self.stores.contains(&id) {
            self.storage
                .send(EntityRead::<DataStore>(id))
                .into_actor(self)
                .then(move |res, act, _ctx| {
                    let _ = match res {
                        Ok(Ok(e)) => {
                            if addr.is_some() {
                                let _ =
                                    addr.unwrap().do_send(ReadResult(index, Some(e.1)));
                            } else {
                                act.bridge.do_send(LocalResponseMessage(
                                    act.group_id.clone(),
                                    index,
                                    Some(json!({ "data": String::from_utf8(e.1).unwrap_or(Default::default()) })),
                                ))
                            }
                        },
                        _ => {
                            if addr.is_some() {
                                let _ = addr.unwrap().do_send(ReadResult(index, None));
                            } else {
                                act.bridge.do_send(LocalResponseMessage(
                                    act.group_id.clone(),
                                    index,
                                    Some(json!({ "data": "" })),
                                ))
                            }
                        }
                    };
                    actor_ok(())
                })
                .wait(ctx);
        } else {
            if self.read_tasks.contains_key(&id) {
                self.read_tasks
                    .get(&id)
                    .and_then(|read_addr| Some(read_addr.do_send(AppendTaskMessage(index, addr))));
            } else {
                let mut virtual_bytes = [0u8; PUBLIC_KEY_LENGTH];
                virtual_bytes.copy_from_slice(id.as_ref());

                if let Some((pk, peer_addr, _)) = self.dht.search(&virtual_bytes) {
                    if pk != self.pk.as_bytes() {
                        let read_addr = ReadActor::new(
                            self.group_id.clone(),
                            peer_addr.clone(),
                            self.pk.clone(),
                            self.psk.clone(),
                            id.clone(),
                            self.bridge.clone(),
                            ctx.address(),
                            index,
                            addr,
                        )
                        .start();
                        self.read_tasks.insert(id, read_addr);
                    }
                } else {
                    if addr.is_some() {
                        let _ = addr.unwrap().do_send(ReadResult(index, None));
                    } else {
                        self.bridge.do_send(LocalResponseMessage(
                            self.group_id.clone(),
                            index,
                            Some(json!({ "data": "" })),
                        ))
                    }
                }
            }
        }
    }

    fn write(&mut self, data: Vec<u8>) -> ID {
        let id = ID::new(&data[..]);

        let mut virtual_bytes = [0u8; PUBLIC_KEY_LENGTH];
        virtual_bytes.copy_from_slice(id.as_ref());

        if let Some((pk, peer_addr, _)) = self.dht.search(&virtual_bytes) {
            if pk == self.pk.as_bytes() {
                println!("DEBUG Write Data: {}", id);
                self.storage
                    .do_send(EntityWrite(DataStore(id.clone(), data)));
                self.stores.insert(id.clone());
            } else {
                self.bridge.do_send(EventMessage(
                    self.group_id.clone(),
                    peer_addr.clone(),
                    bincode::serialize(&Event::new_write(id.clone(), data, &self.pk, &self.psk))
                        .unwrap_or(vec![]),
                ))
            }
        } else {
            println!("DEBUG Write Data: {}", id);
            self.storage
                .do_send(EntityWrite(DataStore(id.clone(), data)));
            self.stores.insert(id.clone());
        }

        id
    }
}

impl Actor for DSActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let self_addr = ctx.address();
        let self_group_id = self.group_id.clone();
        self.bridge.do_send(RegisterBridgeMessage(
            self_group_id.clone(),
            self_group_id,
            self_addr,
        ));

        let configure = Configure::load(None);
        let init_bootstraps = configure.bootstrap_peers;

        for (peer_addr, socket) in init_bootstraps.iter() {
            println!("DEBUG: start bootstrap: {}", peer_addr);
            self.bridge.do_send(PeerJoinMessage(
                self.group_id.clone(),
                peer_addr.clone(),
                vec![],
                Some(socket.clone()),
            ));
        }
    }
}

impl BridgeActor for DSActor {}

/// receive local rpc request from bridge actor, and send to rpc
impl Handler<LocalMessage> for DSActor {
    type Result = ();

    fn handle(&mut self, msg: LocalMessage, ctx: &mut Self::Context) -> Self::Result {
        let (_group, index, params, _socket) = (msg.0, msg.1, msg.2, msg.3);

        let method_option = params.get("method").and_then(|m| m.as_str());
        let params_option = params.get("params");
        if method_option.is_some() && params_option.is_some() {
            let method = method_option.unwrap();
            let params = params_option.unwrap();

            match method {
                "write" => {
                    let data_option = params.get("data").and_then(|t| t.as_str());
                    if data_option.is_some() {
                        let id = self.write(data_option.unwrap().as_bytes().to_vec());
                        return self.bridge.do_send(LocalResponseMessage(
                            self.group_id.clone(),
                            index,
                            Some(json!({ "data_id": format!("{}", id) })),
                        ));
                    }
                }
                "read" => {
                    let id_option = params
                        .get("id")
                        .and_then(|t| t.as_str().and_then(|s| ID::from_str(s).ok()));
                    if id_option.is_some() {
                        return self.read(id_option.unwrap(), index, None, ctx);
                    }
                }

                _ => {}
            }
        }

        self.bridge
            .do_send(LocalResponseMessage(self.group_id.clone(), index, None));
    }
}

/// receive send to upper rpc request from bridge actor, and send to rpc
impl Handler<UpperMessage> for DSActor {
    type Result = ();

    fn handle(&mut self, msg: UpperMessage, _ctx: &mut Self::Context) -> Self::Result {
        let (_group, _index, _block_bytes) = (msg.0, msg.1, msg.2);
    }
}

/// receive send to lower rpc request from bridge actor, and send to rpc
impl Handler<LowerMessage> for DSActor {
    type Result = ();

    fn handle(&mut self, msg: LowerMessage, _ctx: &mut Self::Context) -> Self::Result {
        let (_group, _index, _block_bytes) = (msg.0, msg.1, msg.2);
    }
}

/// receive send to lower rpc request from bridge actor, and send to rpc
impl Handler<LevelPermissionMessage> for DSActor {
    type Result = ();

    fn handle(&mut self, msg: LevelPermissionMessage, _ctx: &mut Self::Context) -> Self::Result {
        let (_group, _index, _permission_bytes, _socket) = (msg.0, msg.1, msg.2, msg.3);
    }
}

impl Handler<LocalResponseMessage> for DSActor {
    type Result = ();

    fn handle(&mut self, msg: LocalResponseMessage, _ctx: &mut Self::Context) -> Self::Result {
        let (_group, _index, _params_option) = (msg.0, msg.1, msg.2);
    }
}

impl Handler<UpperResponseMessage> for DSActor {
    type Result = ();

    fn handle(&mut self, msg: UpperResponseMessage, _ctx: &mut Self::Context) -> Self::Result {
        let (_group, _index, _event_id_option) = (msg.0, msg.1, msg.2);
    }
}

impl Handler<LowerResponseMessage> for DSActor {
    type Result = ();

    fn handle(&mut self, msg: LowerResponseMessage, _ctx: &mut Self::Context) -> Self::Result {
        let (_group, _index, _event_id_option) = (msg.0, msg.1, msg.2);
    }
}

impl Handler<LevelPermissionResponseMessage> for DSActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: LevelPermissionResponseMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let (_group, _index, _is_joined) = (msg.0, msg.1, msg.2);
    }
}

/// receive event message from bridge actor, and send to p2p
impl Handler<EventMessage> for DSActor {
    type Result = ();

    fn handle(&mut self, msg: EventMessage, ctx: &mut Self::Context) {
        let (_group, _peer_addr, event_bytes) = (msg.0, msg.1, msg.2);
        let event_option = bincode::deserialize::<Event>(&event_bytes);
        if event_option.is_err() {
            return;
        }

        self.handle_event_type(event_option.unwrap().event_type, ctx);
    }
}

/// receive peer join message from bridge actor, and send to p2p
impl Handler<PeerJoinMessage> for DSActor {
    type Result = ();

    fn handle(&mut self, msg: PeerJoinMessage, _ctx: &mut Self::Context) -> Self::Result {
        let (group, peer_addr, _join_bytes, _socket_option) = (msg.0, msg.1, msg.2, msg.3);

        self.dht.add(peer_addr.to_bytes(), peer_addr.clone());

        // let _ = bincode::deserialize::<PublicKey>(&join_bytes)
        //     .map(|pk| self.dht.add(pk.to_bytes(), peer_addr.clone()));

        self.bridge
            .do_send(PeerJoinResultMessage(group, peer_addr, true, vec![]));
    }
}

/// receive peer join message from bridge actor, and send to p2p
impl Handler<PeerJoinResultMessage> for DSActor {
    type Result = ();

    fn handle(&mut self, msg: PeerJoinResultMessage, _ctx: &mut Self::Context) -> Self::Result {
        let (_group, peer_addr, is_joined, _help_peers) = (msg.0, msg.1, msg.2, msg.3);
        if is_joined {
            self.dht.add(peer_addr.to_bytes(), peer_addr);
        }
    }
}

/// receive peer leave message from bridge actor, and send to p2p
impl Handler<PeerLeaveMessage> for DSActor {
    type Result = ();

    fn handle(&mut self, msg: PeerLeaveMessage, _ctx: &mut Self::Context) -> Self::Result {
        let (_group, peer_addr, _is_leave_force) = (msg.0, msg.1, msg.2);
        self.dht.remove(peer_addr.as_bytes());
    }
}

impl Handler<Read> for DSActor {
    type Result = ();

    fn handle(&mut self, msg: Read, ctx: &mut Self::Context) -> Self::Result {
        let (addr, index, id) = (msg.0, msg.1, msg.2);
        self.read(id, index, Some(addr), ctx);
    }
}

impl Handler<Write> for DSActor {
    type Result = ();

    fn handle(&mut self, msg: Write, _ctx: &mut Self::Context) -> Self::Result {
        let (addr, index, data) = (msg.0, msg.1, msg.2);
        let id = self.write(data);
        let _ = addr.do_send(WriteResult(index, Some(id)));
    }
}

impl Handler<Drop> for DSActor {
    type Result = ();

    fn handle(&mut self, msg: Drop, _ctx: &mut Self::Context) -> Self::Result {
        let _id = msg.0;
    }
}

impl Handler<ReadCloseMessage> for DSActor {
    type Result = ();

    fn handle(&mut self, msg: ReadCloseMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.read_tasks.remove(&msg.0);
    }
}
