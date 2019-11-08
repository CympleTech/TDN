use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use crate::actor::prelude::*;
use crate::crypto::hash::H256;
use crate::crypto::keypair::{PrivateKey, PublicKey};
use crate::primitives::functions::get_default_storage_path;
use crate::primitives::functions::{try_resend_times, DEFAULT_TIMES};
use crate::primitives::types::GroupID;
use crate::storage::{DiskStorageActor, Entity, EntityRead, EntityWrite};
use crate::traits::actor::P2PBridgeActor;
use crate::traits::message::p2p_message::*;

use super::codec::P2PHead;
use super::content::P2PContent;
use super::dht::DHTTable;
use super::session::{P2PAddrMessage, P2PMessage, P2PSessionActor};

/// p2p actor service.
/// it will handle every event message and p2p peer.
/// outside use donot need care how to send, only care send to who.
#[derive(Clone)]
pub struct P2PActor<A: P2PBridgeActor> {
    version: u16,
    psk: PrivateKey,
    pk: PublicKey,
    bridge: Option<Addr<A>>,
    storage: Addr<DiskStorageActor>,
    tables: HashMap<GroupID, DHTTable>,
    session: Addr<P2PSessionActor<A>>,
    holepunching: HashMap<PublicKey, (Instant, SocketAddr, GroupID, Vec<P2PMessage>)>,
}

impl<A: P2PBridgeActor> P2PActor<A> {
    pub fn load(session: Addr<P2PSessionActor<A>>, psk: Option<PrivateKey>) -> Self {
        let psk = if psk.is_none() {
            // TODO load from storage or generate
            PrivateKey::generate()
        } else {
            psk.unwrap()
        };
        let pk = psk.generate_public_key();
        let mut path = get_default_storage_path();
        path.push("p2p");
        path.push(format!("{}", pk));

        let storage = DiskStorageActor::run(Some(path));

        // load psk and tables
        Self {
            version: 1u16,
            psk: psk,
            pk: pk,
            bridge: None,
            storage: storage,
            tables: HashMap::new(), // load
            session: session,
            holepunching: HashMap::new(),
        }
    }

    /// try send received event to bridge actor
    fn send_bridge<M: 'static>(&self, message: M)
    where
        A: Handler<M>,
        M: Message + Send + Clone,
        <M as Message>::Result: Send,
        <A as Actor>::Context: ToEnvelope<A, M>,
    {
        if self.bridge.is_none() {
            //
        } else {
            let _ = try_resend_times(self.bridge.clone().unwrap(), message, DEFAULT_TIMES)
                .map_err(|_| println!("Send Message to bridge fail"));
        }
    }

    /// try send received event to bridge actor
    fn send_session<M: 'static>(&self, message: M)
    where
        P2PSessionActor<A>: Handler<M>,
        M: Message + Send + Clone,
        <M as Message>::Result: Send,
        <P2PSessionActor<A> as Actor>::Context: ToEnvelope<P2PSessionActor<A>, M>,
    {
        let _ = try_resend_times(self.session.clone(), message, DEFAULT_TIMES)
            .map_err(|_| println!("Send Message to udp session fail"));
    }

    fn new_p2p_message(
        &self,
        group: GroupID,
        to: PublicKey,
        socket: SocketAddr,
        content: P2PContent,
    ) -> P2PMessage {
        let mut head = P2PHead::new(self.version, group, self.pk.clone(), to);
        let hash_content = H256::new(&bincode::serialize(&content).unwrap_or(vec![])[..]);
        head.update_signature(self.psk.sign_bytes(&hash_content.to_vec()));
        P2PMessage(head, content, socket)
    }

    /// Timed task, include: Heart Beat and NAT holepunching
    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::new(5, 0), |act, ctx| {
            let mut send_peer_leave: Vec<ReceivePeerLeaveMessage> = vec![];
            let mut send_heartbeat: Vec<(GroupID, PublicKey, SocketAddr, P2PContent)> = vec![];
            for (group, table) in act.tables.iter_mut() {
                let (mut next, mut dis) = table.next_hb_peers();
                for _ in 0..dis.len() {
                    if let Some(id) = dis.pop() {
                        table.remove_peer(&id);
                        send_peer_leave.push(ReceivePeerLeaveMessage(group.clone(), id, false));
                    }
                }

                while !next.is_empty() {
                    let (pk, socket) = next.pop().unwrap();
                    send_heartbeat.push((group.clone(), pk, socket, P2PContent::HeartBeat));
                }
            }

            while !send_peer_leave.is_empty() {
                let message = send_peer_leave.pop().unwrap();
                act.send_bridge(message);
            }

            while !send_heartbeat.is_empty() {
                let (group, to, socket, content) = send_heartbeat.pop().unwrap();
                let message = act.new_p2p_message(group, to, socket, content);
                act.send_session(message);
            }

            // check nat hole punching
            let mut need_delete: Vec<PublicKey> = act
                .holepunching
                .iter()
                .filter_map(|(pk, (ins, socket, group, _tasks))| {
                    if Instant::now().duration_since(ins.clone()) > Duration::new(8, 0) {
                        println!("Again hole punching {} : {:?}", pk, socket);
                        act.send_session(act.new_p2p_message(
                            group.clone(),
                            pk.clone(),
                            socket.clone(),
                            P2PContent::HolePunching,
                        ));
                        Some(pk)
                    } else {
                        None
                    }
                })
                .cloned()
                .collect();

            while !need_delete.is_empty() {
                let pk = need_delete.pop().unwrap();
                act.holepunching.remove(&pk);
            }

            act.hb(ctx);
        });
    }
}

/// impl Actor for NetworkBridgeActor
impl<A: P2PBridgeActor> Actor for P2PActor<A> {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let _ = try_resend_times(
            self.session.clone(),
            P2PAddrMessage(ctx.address()),
            DEFAULT_TIMES,
        )
        .map_err(|_| println!("Send p2p addr to session fail"));

        DHTTableStore::async_load(&self.pk, &self.storage, self, ctx);

        self.hb(ctx);
    }
}

impl<A: P2PBridgeActor> Handler<P2PBridgeAddrMessage<A>> for P2PActor<A> {
    type Result = ();

    fn handle(&mut self, msg: P2PBridgeAddrMessage<A>, _ctx: &mut Self::Context) -> Self::Result {
        self.bridge = Some(msg.0);
        // load saved tables
    }
}

impl<A: P2PBridgeActor> P2PBridgeActor for P2PActor<A> {}

impl<A: P2PBridgeActor> Handler<ReceiveEventMessage> for P2PActor<A> {
    type Result = ();

    fn handle(&mut self, msg: ReceiveEventMessage, _ctx: &mut Self::Context) -> Self::Result {
        let (group, peer_addr, event) = (msg.0, msg.1, msg.2);
        if peer_addr == self.pk {
            return self.send_bridge(ReceiveEventMessage(group, peer_addr, event));
        }

        if let Some(table) = self.tables.get(&group) {
            if self.holepunching.contains_key(&peer_addr) {
                let socket = self
                    .holepunching
                    .get(&peer_addr)
                    .as_ref()
                    .unwrap()
                    .1
                    .clone();
                let message = self.new_p2p_message(
                    group,
                    peer_addr.clone(),
                    socket,
                    P2PContent::Event(event),
                );
                self.holepunching
                    .get_mut(&peer_addr)
                    .unwrap()
                    .3
                    .push(message);
            } else if let Some(socket) = table.get_socket_addr(&peer_addr) {
                self.send_session(self.new_p2p_message(
                    group,
                    peer_addr,
                    socket,
                    P2PContent::Event(event),
                ));
            }
        }
    }
}

impl<A: P2PBridgeActor> Handler<ReceivePeerJoinMessage> for P2PActor<A> {
    type Result = ();

    fn handle(&mut self, msg: ReceivePeerJoinMessage, _ctx: &mut Self::Context) -> Self::Result {
        // join group to p2p
        let (group, peer_addr, result, socket_addr) = (msg.0, msg.1, msg.2, msg.3);
        if !self.tables.contains_key(&group) {
            self.tables.insert(group.clone(), DHTTable::new(&self.pk));
        }

        if let Some(table) = self.tables.get_mut(&group) {
            println!("DEBUG: start peer join: {}", peer_addr);
            table.add_tmp_peer(&peer_addr, socket_addr);
            if self.holepunching.contains_key(&peer_addr) {
                let socket = self
                    .holepunching
                    .get(&peer_addr)
                    .as_ref()
                    .unwrap()
                    .1
                    .clone();
                let message = self.new_p2p_message(
                    group,
                    peer_addr.clone(),
                    socket,
                    P2PContent::Join(result),
                );
                self.holepunching
                    .get_mut(&peer_addr)
                    .unwrap()
                    .3
                    .push(message);
            } else if let Some(socket) = table.get_socket_addr(&peer_addr) {
                self.send_session(self.new_p2p_message(
                    group,
                    peer_addr,
                    socket,
                    P2PContent::Join(result),
                ));
            }
        }
    }
}

impl<A: P2PBridgeActor> Handler<ReceivePeerJoinResultMessage> for P2PActor<A> {
    type Result = ();

    fn handle(
        &mut self,
        msg: ReceivePeerJoinResultMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let (group, peer_addr, result, helps) = (msg.0, msg.1, msg.2, msg.3);
        println!("DEBUG: peer join: {}", result);
        let need_store = if let Some(table) = self.tables.get_mut(&group) {
            let mut need_store = false;
            if let Some(socket) = table.get_socket_addr(&peer_addr) {
                if result {
                    if table.fixed_peer(&peer_addr) {
                        need_store = true;
                    }

                    let dht = helps
                        .iter()
                        .filter_map(|peer_addr| {
                            if let Some(addr) = table.get_socket_addr(peer_addr) {
                                Some((peer_addr.clone(), addr))
                            } else {
                                None
                            }
                        })
                        .collect();

                    self.send_session(self.new_p2p_message(
                        group,
                        peer_addr,
                        socket,
                        P2PContent::DHT(dht),
                    ));
                } else {
                    self.send_session(self.new_p2p_message(
                        group,
                        peer_addr,
                        socket,
                        P2PContent::Leave,
                    ));
                }
            }
            need_store
        } else {
            false
        };

        if need_store {
            DHTTableStore::async_store(self.pk.clone(), self.tables.clone(), &self.storage);
        }
    }
}

impl<A: P2PBridgeActor> Handler<ReceivePeerLeaveMessage> for P2PActor<A> {
    type Result = ();

    fn handle(&mut self, msg: ReceivePeerLeaveMessage, _ctx: &mut Self::Context) -> Self::Result {
        let (group, peer_addr, is_force) = (msg.0, msg.1, msg.2);
        let socket = if let Some(table) = self.tables.get_mut(&group) {
            if is_force {
                let socket = table.get_socket_addr(&peer_addr);
                table.remove_peer(&peer_addr);
                socket
            } else {
                // do some other connect
                None
            }
        } else {
            None
        };

        if let Some(socket) = socket {
            self.send_session(self.new_p2p_message(group, peer_addr, socket, P2PContent::Leave));
        }
    }
}

/// handle receive P2PMessage from UDP
impl<A: P2PBridgeActor> Handler<P2PMessage> for P2PActor<A> {
    type Result = ();

    fn handle(&mut self, msg: P2PMessage, _ctx: &mut Self::Context) -> Self::Result {
        let (head, content, socket) = (msg.0, msg.1, msg.2);
        let (group, from, to) = (head.gid, head.from, head.to);

        // check self group
        if !self.tables.contains_key(&group) {
            return;
        }

        let table = self.tables.get_mut(&group).unwrap();

        // check version include

        // TODO check if send to is self node
        // if true handle it, if not send to closest peer
        if from == self.pk || to != self.pk {
            return;
        }

        // remove from hole punching
        if self.holepunching.contains_key(&from) {
            if let Some((_, _, _, mut tasks)) = self.holepunching.remove(&from) {
                loop {
                    if let Some(message) = tasks.pop() {
                        // use self.send_session will wrong ?
                        let _ = try_resend_times(self.session.clone(), message, DEFAULT_TIMES)
                            .map_err(|_| println!("Send Message to udp session fail"));
                    } else {
                        break;
                    }
                }
            }
        }

        match content {
            P2PContent::HeartBeat => {
                table.update_hb_peers(&from);
                self.send_session(self.new_p2p_message(
                    group,
                    from,
                    socket,
                    P2PContent::HeartBeatOk,
                ));
            }
            P2PContent::HeartBeatOk => {
                table.update_hb_peers(&from);
            }
            P2PContent::DHT(mut pk_sockets) => {
                println!("DEBUG: receive DHT {}", from);
                if table.fixed_peer(&from) {
                    DHTTableStore::async_store(self.pk.clone(), self.tables.clone(), &self.storage);
                }

                let pks = pk_sockets.iter().map(|(pk, _)| pk.clone()).collect();
                self.send_bridge(ReceivePeerJoinResultMessage(group.clone(), from, true, pks));
                loop {
                    if let Some((other_pk, socket_addr)) = pk_sockets.pop() {
                        let not_contain = self
                            .tables
                            .get(&group)
                            .and_then(|t| Some(t.check_add(&other_pk)));
                        if let Some(true) = not_contain {
                            self.tables
                                .get_mut(&group)
                                .map(|t| t.add_tmp_peer(&other_pk, None));

                            println!("DEBUG: start hole punching {}, {}", other_pk, socket_addr);
                            self.send_session(self.new_p2p_message(
                                group.clone(),
                                other_pk.clone(),
                                socket_addr,
                                P2PContent::HolePunching,
                            ));

                            self.holepunching
                                .entry(other_pk)
                                .and_modify(|time| {
                                    time.0 = Instant::now();
                                })
                                .or_insert((Instant::now(), socket_addr, group.clone(), vec![]));
                        }
                    } else {
                        break;
                    }
                }
            }
            P2PContent::Hole(pk, socket_addr) => {
                if table.check_add(&from) {
                    println!("DEBUG: need hole punching : {}", socket);
                    self.send_session(self.new_p2p_message(
                        group,
                        pk,
                        socket_addr,
                        P2PContent::HolePunching,
                    ));
                }
            }
            P2PContent::HolePunching => {
                println!("DEBUG: success hole punching : {}", socket);
                table.fixed_tmp_peer(&from, socket);
                self.send_session(self.new_p2p_message(
                    group,
                    from,
                    socket,
                    P2PContent::HolePunchingOk,
                ));
            }

            P2PContent::HolePunchingOk => {
                println!("DEBUG: success hole punching : {}", socket);
                table.fixed_tmp_peer(&from, socket);
            }

            P2PContent::Leave => {
                println!("DEBUG: receive peer leave: {}", from);
                table.remove_peer(&from);
                self.send_bridge(ReceivePeerLeaveMessage(group, from, true));
            }
            P2PContent::Join(join_bytes) => {
                println!("DEBUG: receive peer join: {}", from);
                table.add_tmp_peer(&from, Some(socket));
                self.send_bridge(ReceivePeerJoinMessage(
                    group,
                    from,
                    join_bytes,
                    Some(socket),
                ));
            }
            P2PContent::Event(event_bytes) => {
                if table.contains(&from) {
                    self.send_bridge(ReceiveEventMessage(group, from, event_bytes));
                }
            }
            _ => {}
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct DHTTableStore(PublicKey, HashMap<GroupID, DHTTable>);

impl Entity for DHTTableStore {
    type Key = String;

    fn key(&self) -> Self::Key {
        format!("{}", self.0)
    }
}

impl DHTTableStore {
    pub fn async_store(
        pk: PublicKey,
        table: HashMap<GroupID, DHTTable>,
        addr: &Addr<DiskStorageActor>,
    ) {
        let _ = try_resend_times(
            addr.clone(),
            EntityWrite(DHTTableStore(pk, table)),
            DEFAULT_TIMES,
        )
        .map_err(|_| println!("Send to storage fail"));
    }

    pub fn async_load<A: P2PBridgeActor>(
        pk: &PublicKey,
        addr: &Addr<DiskStorageActor>,
        p2p_actor: &P2PActor<A>,
        ctx: &mut <P2PActor<A> as Actor>::Context,
    ) {
        let storage_addr = addr.clone();
        storage_addr
            .send(EntityRead::<DHTTableStore>(format!("{}", pk)))
            .into_actor(p2p_actor)
            .then(move |res, act, _ctx| {
                match res {
                    Ok(Ok(e)) => act.tables = e.1,
                    _ => {}
                }

                actor_ok(())
            })
            .wait(ctx);
    }

    pub fn _async_delete(pk: PublicKey, _addr: &Addr<DiskStorageActor>) {
        println!("DEBUG: async delete tables: {}", pk);
    }
}
