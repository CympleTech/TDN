use std::collections::BTreeSet;
use std::collections::HashMap;
use std::time::Duration as StdDuration;
use time::{Duration, Timespec};

use core::actor::prelude::*;
//use core::crypto::hash::H256;
use core::primitives::functions::get_default_storage_path;
use core::primitives::functions::{try_resend_times, DEFAULT_TIMES};
use core::primitives::types::EventID;
use core::storage::{DiskDatabase, DiskStorageActor, EntityRead, EntityWrite};
use core::traits::propose::Event as EventTrait;
use core::traits::propose::Message as MessageTrait;
use core::traits::propose::Peer;

use crate::block::{Block, BlockID};
use crate::event::Event;
use crate::fixed_chain::FixedChain;
use crate::leader::calculate_leader;
use crate::message::Message;
use crate::store::{BlockStore, ChainStore};
use crate::traits::{
    HandleEventMessage, HandleEventResultMessage, HandlePeerInMessage, HandlePeerOutMessage,
    QuickPBFTBridgeActor,
};

const MIN_BLOCK_SECOND: i64 = 5;
const MAX_BLOCK_SECOND: i64 = 20;

pub struct QuickPBFTActor<
    M: 'static + MessageTrait,
    P: 'static + Peer,
    A: QuickPBFTBridgeActor<M, P>,
> {
    actor: Addr<A>,
    storage: Addr<DiskStorageActor>,
    peers: BTreeSet<P::PublicKey>,

    psk: P::PrivateKey,
    pk: P::PublicKey,

    //merkle_root: H256,
    chain: FixedChain,
    blocker: P::PublicKey,
    last_block_timestamp: Timespec,
    last_block_height: u64,

    sync_status: bool,
    sync_chain: Vec<BlockID>,
    sync_height: u64,

    blocks: HashMap<BlockID, Block<M, P>>,
    events: HashMap<EventID, Event<M, P>>,

    votes: HashMap<BlockID, Vec<EventID>>,
    verify: HashMap<BlockID, Vec<P::PublicKey>>,
    commit: HashMap<BlockID, Vec<P::PublicKey>>,

    leader_confirm: HashMap<P::PublicKey, Vec<P::PublicKey>>,
}

impl<M: 'static + MessageTrait, P: 'static + Peer, A: QuickPBFTBridgeActor<M, P>>
    QuickPBFTActor<M, P, A>
{
    /// load pbft from storage or generate
    pub fn load(
        pk: P::PublicKey,
        psk: P::PrivateKey,
        actor: Addr<A>,
        peers: Vec<P::PublicKey>,
    ) -> Self {
        let mut path = get_default_storage_path();
        path.push("quick_pbft");
        path.push(format!("{}", pk));
        let db = DiskDatabase::new(Some(path.clone()));

        let chain = if let Ok(chain) = db.read_entity::<ChainStore<P>>(pk.to_string()) {
            chain.1
        } else {
            vec![]
        };

        let last_block_height = if let Ok(block_store) = db.read_entity::<BlockStore<M, P>>(
            chain
                .iter()
                .last()
                .and_then(|b| Some(b.clone()))
                .unwrap_or(BlockID::default()),
        ) {
            block_store.0.height()
        } else {
            0
        };

        drop(db);

        let storage = DiskStorageActor::run(Some(path));

        Self {
            actor: actor,
            storage: storage,
            peers: peers.iter().map(|p| p).cloned().collect(),

            psk: psk,
            pk: pk.clone(),

            chain: FixedChain::new(chain),
            blocker: pk,
            last_block_timestamp: time::now_utc().to_timespec(),
            last_block_height: last_block_height,

            sync_status: false,
            sync_chain: vec![],
            sync_height: last_block_height,

            blocks: HashMap::new(),
            events: HashMap::new(),

            votes: HashMap::new(),
            verify: HashMap::new(),
            commit: HashMap::new(),

            leader_confirm: HashMap::new(),
        }
    }

    /// try send event result to bridge actor
    fn send_bridge(&self, message: HandleEventResultMessage<M, P>) {
        let _ = try_resend_times(self.actor.clone(), message, DEFAULT_TIMES)
            .map_err(|_| println!("Send Message to network fail"));
    }

    fn heart_beat(&self, ctx: &mut Context<Self>) {
        ctx.run_later(StdDuration::new(MAX_BLOCK_SECOND as u64, 0), |act, ctx| {
            let (is_expired, new_pk_option) = act.is_leader_expired();

            if is_expired && new_pk_option.is_some() {
                act.build_p2p_leader_expired(new_pk_option.unwrap());
            } else if is_expired {
                act.lead_blocker();
                println!("DEBUG: send heart beat :{}", act.blocker);
                act.send_bridge(HandleEventResultMessage(
                    vec![(
                        act.blocker.clone(),
                        Event::new_from_self_message(act.pk.clone(), Message::HeartBeat, &act.psk),
                    )],
                    None,
                ));
            }

            if act.sync_status {
                println!("DEBUG: start sync again");
                let block_id = act.sync_chain.iter().last();
                let event = if block_id.is_none() {
                    Event::new_from_self_message(
                        act.pk.clone(),
                        Message::Sync(act.sync_height),
                        &act.psk,
                    )
                } else {
                    Event::new_from_self_message(
                        act.pk.clone(),
                        Message::BlockReq(block_id.unwrap().clone()),
                        &act.psk,
                    )
                };

                act.send_bridge(HandleEventResultMessage(
                    act.peers
                        .iter()
                        .filter_map(|p| {
                            if p != &act.pk {
                                Some((p.clone(), event.clone()))
                            } else {
                                None
                            }
                        })
                        .collect(),
                    None,
                ));
            }

            act.heart_beat(ctx);
        });
    }

    fn new_event(&self, message: Message<M, P>) -> Event<M, P> {
        Event::new_from_self_message(self.pk.clone(), message, &self.psk)
    }
}

impl<M: 'static + MessageTrait, P: 'static + Peer, A: QuickPBFTBridgeActor<M, P>>
    QuickPBFTActor<M, P, A>
{
    fn create_sync_request(&mut self) -> Vec<(P::PublicKey, Event<M, P>)> {
        self.build_p2p(
            &self.pk.clone(),
            &Event::new_from_self_message(
                self.pk.clone(),
                Message::Sync(self.last_block_height),
                &self.psk,
            ),
        )
    }

    fn handle_event(
        &mut self,
        sender: P::PublicKey,
        event: Event<M, P>,
        ctx: &mut <Self as Actor>::Context,
    ) -> (
        Vec<(P::PublicKey, Event<M, P>)>, // p2p
        Option<Block<M, P>>,              // confirm block
    ) {
        let event_creator = &event.creator;

        // Guard
        match event.message {
            Message::Tx(_) => {
                if self.events.contains_key(event.id()) {
                    return (vec![], None);
                }
                self.events.insert(event.id().clone(), event.clone());
            }
            Message::Block(ref block) => {
                self.lead_blocker();

                let block_id = block.id();
                if self.blocks.contains_key(&block_id) {
                    return (vec![], None);
                }

                if block.height() <= self.last_block_height {
                    return (vec![], None);
                }

                if let Some(lastest) = self.chain.lastest() {
                    if block.previous() != lastest {
                        return (vec![], None);
                    }
                }

                if !self.blocks.is_empty() {
                    let key = self.blocks.keys().into_iter().next().unwrap().clone();
                    if &self.blocker != self.blocks.get(&key).unwrap().blocker() {
                        self.blocks.remove(&key);
                        self.votes.remove(&key);
                        self.commit.remove(&key);
                        self.verify.remove(&key);
                    }
                }

                if &self.blocker != block.blocker() {
                    println!(
                        "DEBUG: peers len: {}, must: {}, block: {} !!!!!!!!!",
                        self.peers.len(),
                        self.blocker,
                        block.id()
                    );
                    return (vec![], None);
                }

                if !self.votes.contains_key(block_id) {
                    self.votes
                        .insert(block_id.clone(), vec![event.id().clone()]);
                } else {
                    if self.votes.get(block_id).unwrap().contains(event.id()) {
                        return (vec![], None);
                    } else {
                        self.votes
                            .get_mut(block_id)
                            .unwrap()
                            .push(event.id().clone());
                    }
                }
            }
            Message::Commit(ref block_id) | Message::Verify(ref block_id) => {
                if self.chain.contains(&block_id) {
                    return (vec![], None);
                }

                if !self.votes.contains_key(block_id) {
                    self.votes
                        .insert(block_id.clone(), vec![event.id().clone()]);
                } else {
                    if self.votes.get(block_id).unwrap().contains(event.id()) {
                        return (vec![], None);
                    } else {
                        self.votes
                            .get_mut(block_id)
                            .unwrap()
                            .push(event.id().clone());
                    }
                }
            }
            Message::Leader(ref pk, _index) => {
                if self.peers.contains(&pk) {
                    if let (true, _) = self.is_leader_expired() {
                        self.leader_confirm
                            .entry(pk.clone())
                            .and_modify(|peers| {
                                if !peers.contains(&pk) {
                                    peers.push(sender.clone());
                                }
                            })
                            .or_insert(vec![sender.clone(), self.pk.clone()]);
                    }
                }
            }
            _ => {}
        }

        match &event.message {
            Message::Block(block) => {
                println!("DEBUG: PBFT receive block create: {}", block.id());
                let block_id = block.id();

                self.blocks.insert(block_id.clone(), block.clone());
                let mut pks = vec![event_creator.clone(), self.pk.clone()];
                self.verify
                    .entry(block_id.clone())
                    .and_modify(|peers| peers.append(&mut pks))
                    .or_insert(pks);

                let mut p2p = vec![];
                p2p.append(&mut self.build_p2p_verify(&block_id));
                let (mut build_commit, confirm_block) = self.build_p2p_commit(&block_id);
                p2p.append(&mut build_commit);

                return (p2p, confirm_block);
            }
            Message::Verify(block_id) => {
                println!("DEBUG: PBFT receive block verify: {}", block_id);
                self.verify
                    .entry(block_id.clone())
                    .and_modify(|peers| peers.push(event_creator.clone()))
                    .or_insert(vec![event_creator.clone()]);

                let peer_len = self.peers.len();
                let (next_p2p, confirm_block) = if (peer_len < 3
                    && peer_len == self.verify.get(&block_id).unwrap().len())
                    || 2 * (self.peers.len() + 1) <= 3 * self.verify.get(&block_id).unwrap().len()
                {
                    let mut pk = vec![self.pk.clone()];
                    self.commit
                        .entry(block_id.clone())
                        .and_modify(|peers| peers.append(&mut pk))
                        .or_insert(pk);

                    // TODO remove verified block
                    let mut p2p = vec![];
                    let (mut build_commit, confirm_block) = self.build_p2p_commit(&block_id);
                    p2p.append(&mut build_commit);

                    return (p2p, confirm_block);
                } else {
                    (vec![], None)
                };

                (next_p2p, confirm_block)
            }
            Message::Commit(block_id) => {
                println!("DEBUG: PBFT receive block commit: {}", block_id);
                self.commit
                    .entry(block_id.clone())
                    .and_modify(|peers| peers.push(event_creator.clone()))
                    .or_insert(vec![event_creator.clone()]);

                let replied = self.confirm_block(&block_id);
                (vec![], replied)
            }
            Message::Tx(_) => {
                let mut p2p = self.build_p2p(&sender, &event);
                let (mut build_blocks, has_block) = self.build_p2p_block();
                let confirm_block = if let Some(block_id) = has_block {
                    p2p.append(&mut build_blocks);
                    p2p.append(&mut self.build_p2p_verify(&block_id));
                    let (mut build_commit, confirm_block) = self.build_p2p_commit(&block_id);
                    p2p.append(&mut build_commit);
                    confirm_block
                } else {
                    None
                };

                return (p2p, confirm_block);
            }

            Message::Sync(height) => {
                let now_last_block_option = self.chain.iter().last();
                println!(
                    "DEBUG: Receive Sync, now block: {:?}",
                    now_last_block_option
                );
                if now_last_block_option.is_none() || *height > self.last_block_height {
                    return (
                        vec![(
                            sender.clone(),
                            Event::new_from_self_message(
                                self.pk.clone(),
                                Message::BlockRes(None),
                                &self.psk,
                            ),
                        )],
                        None,
                    );
                }

                let now_last_block_id = now_last_block_option.unwrap().clone();

                self.storage
                    .clone()
                    .send(EntityRead::<BlockStore<M, P>>(now_last_block_id))
                    .into_actor(self)
                    .then(move |res, act, _ctx| {
                        let block: Option<Block<M, P>> = match res {
                            Ok(Ok(block)) => Some(block.0),
                            _ => None,
                        };

                        act.send_bridge(HandleEventResultMessage(
                            vec![(sender.clone(), act.new_event(Message::BlockRes(block)))],
                            None,
                        ));

                        actor_ok(())
                    })
                    .wait(ctx);

                (vec![], None)
            }
            Message::BlockReq(block_id) => {
                println!("DEBUG: Receive Sync Block: {}", block_id);
                self.storage
                    .clone()
                    .send(EntityRead::<BlockStore<M, P>>(block_id.clone()))
                    .into_actor(self)
                    .then(move |res, act, _ctx| {
                        let block: Option<Block<M, P>> = match res {
                            Ok(Ok(block)) => Some(block.0),
                            _ => None,
                        };

                        act.send_bridge(HandleEventResultMessage(
                            vec![(sender.clone(), act.new_event(Message::BlockRes(block)))],
                            None,
                        ));

                        actor_ok(())
                    })
                    .wait(ctx);

                (vec![], None)
            }
            Message::BlockRes(block_option) => {
                println!(
                    "DEBUG: Receive Sync Response: {:?} {}",
                    block_option.as_ref().and_then(|b| Some(b.id())),
                    sender,
                );

                if block_option.is_none() {
                    self.sync_status = false;

                    let need_save = !self.sync_chain.is_empty();

                    while !self.sync_chain.is_empty() {
                        let block_id = self.sync_chain.pop().unwrap();
                        self.chain.push(block_id);
                    }

                    self.lead_blocker();

                    if need_save {
                        self.storage.clone().do_send(EntityWrite(ChainStore::<P>(
                            self.pk.clone(),
                            self.chain.vec().clone(),
                        )));
                    }

                    return (vec![], None);
                }

                let block = block_option.as_ref().unwrap().clone();

                if block.height() <= self.sync_height {
                    return (vec![], None);
                }

                self.sync_status = true;

                self.sync_chain.push(block.id().clone());
                if block.height() > self.last_block_height {
                    self.last_block_height = block.height();
                }

                self.storage
                    .clone()
                    .do_send(EntityWrite(BlockStore::<M, P>(block.clone())));

                let next_p2p = if self.sync_height + 1 == block.height()
                    || block.previous() == &BlockID::default()
                {
                    self.sync_status = false;
                    let need_save = !self.sync_chain.is_empty();

                    while !self.sync_chain.is_empty() {
                        let block_id = self.sync_chain.pop().unwrap();
                        self.chain.push(block_id);
                    }

                    if need_save {
                        self.storage.clone().do_send(EntityWrite(ChainStore::<P>(
                            self.pk.clone(),
                            self.chain.vec().clone(),
                        )));
                    }

                    self.lead_blocker();

                    vec![]
                } else {
                    vec![(
                        sender.clone(),
                        self.new_event(Message::BlockReq(block.previous().clone())),
                    )]
                };

                return (next_p2p, Some(block));
            }
            Message::Leader(pk, _index) => {
                if !self.leader_confirm.contains_key(pk) {
                    return (vec![], None);
                }

                let p2p = if 2 * (self.peers.len() + 1)
                    <= 3 * self.leader_confirm.get(pk).unwrap().len()
                {
                    self.blocker = pk.clone();
                    if let Some(block) = self.create_block() {
                        self.verify
                            .insert(block.id().clone(), vec![self.pk.clone()]);

                        self.blocks.insert(block.id().clone(), block.clone());
                        self.peers
                            .iter()
                            .filter_map(|pk| {
                                if pk != &self.pk {
                                    Some((
                                        pk.clone(),
                                        Event::new_from_self_message(
                                            self.pk.clone(),
                                            Message::Block(block.clone()),
                                            &self.psk,
                                        ),
                                    ))
                                } else {
                                    None
                                }
                            })
                            .collect()
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                };
                (p2p, None)
            }
            Message::HeartBeat => {
                let mut p2p = vec![];
                if self.blocks.is_empty() {
                    let (mut build_blocks, has_block) = self.build_p2p_block();
                    let confirm_block = if let Some(block_id) = has_block {
                        p2p.append(&mut build_blocks);
                        p2p.append(&mut self.build_p2p_verify(&block_id));
                        let (mut build_commit, confirm_block) = self.build_p2p_commit(&block_id);
                        p2p.append(&mut build_commit);
                        confirm_block
                    } else {
                        None
                    };

                    return (p2p, confirm_block);
                } else {
                    (
                        self.blocks
                            .iter()
                            .map(|(_, block)| {
                                (
                                    sender.clone(),
                                    Event::new_from_self_message(
                                        self.pk.clone(),
                                        Message::Block(block.clone()),
                                        &self.psk,
                                    ),
                                )
                            })
                            .collect(),
                        None,
                    )
                }
            }
        }
    }

    fn lead_blocker(&mut self) {
        let last_block = self.chain.lastest();
        let calculate_block = if last_block.is_some() {
            vec![last_block.unwrap().clone()]
        } else {
            vec![]
        };

        self.blocker = if let Some(pk) = calculate_leader::<P>(&self.peers, &calculate_block) {
            pk
        } else {
            self.pk.clone()
        };
    }

    fn create_block(&mut self) -> Option<Block<M, P>> {
        if self.sync_status {
            return None;
        }

        if self.blocker != self.pk {
            return None;
        }

        if self.peers.len() < 2 && self.events.len() == 0 {
            return None;
        }

        if let Some((_, block)) = self.blocks.iter().nth(0) {
            return Some(block.clone());
        }

        let had_time = time::now_utc().to_timespec() - self.last_block_timestamp;

        if (self.events.len() > 0 && had_time > Duration::seconds(MIN_BLOCK_SECOND))
            || (had_time > Duration::seconds(MAX_BLOCK_SECOND))
        {
            self.lead_blocker();
            if self.blocker != self.pk {
                return None; // Add blocker security
            }

            let keys: Vec<EventID> = self.events.keys().cloned().collect();

            let events = keys
                .iter()
                .filter_map(|i| {
                    let event = self.events.remove(i).unwrap();
                    if event.message.is_effective() {
                        Some(event)
                    } else {
                        None
                    }
                })
                .collect();

            let previous_block = self
                .chain
                .iter()
                .last()
                .and_then(|b| Some(b.clone()))
                .unwrap_or(BlockID::default());

            let block = Block::new(
                self.pk.clone(),
                &self.psk,
                events,
                previous_block,
                self.last_block_height + 1,
            );

            println!(
                "DEBUG: create block: {}, now peer: {}",
                block.id(),
                self.peers.len()
            );
            Some(block)
        } else {
            None
        }
    }

    fn confirm_block(&mut self, block_id: &BlockID) -> Option<Block<M, P>> {
        let peers_len = self.peers.len();
        if self.commit.get(&block_id).is_none() {
            return None;
        }

        if (peers_len < 3 && peers_len == self.commit.get(&block_id).unwrap().len())
            || 2 * (peers_len + 1) <= 3 * self.commit.get(&block_id).unwrap().len()
        {
            if self.blocks.contains_key(block_id) {
                self.commit.remove(block_id);
                self.verify.remove(block_id);
                self.votes.remove(block_id);
                let block = self.blocks.remove(&block_id).unwrap();
                for e in block.iter() {
                    self.events.remove(e.id());
                }
                self.chain.push(block_id.clone());

                self.last_block_timestamp = time::now_utc().to_timespec();
                self.last_block_height += 1;

                println!(
                    "DEBUG: now chain length: {}, event: {}",
                    self.last_block_height,
                    block.iter().len()
                );

                // save chain
                self.storage.clone().do_send(EntityWrite(ChainStore::<P>(
                    self.pk.clone(),
                    self.chain.vec().clone(),
                )));

                // save block
                self.storage
                    .clone()
                    .do_send(EntityWrite(BlockStore::<M, P>(block.clone())));

                self.sync_height = self.last_block_height;
                self.lead_blocker();
                println!("DEBUG: Next blocker is {}", self.blocker);

                Some(block)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn is_leader_expired(&self) -> (bool, Option<P::PublicKey>) {
        if !self.peers.contains(&self.blocker) {
            let last_block = self.chain.lastest();
            let calculate_block = if last_block.is_some() {
                vec![last_block.unwrap().clone()]
            } else {
                vec![]
            };

            if let Some(pk) = calculate_leader::<P>(&self.peers, &calculate_block) {
                return (true, Some(pk));
            }
        }

        if time::now_utc().to_timespec() - self.last_block_timestamp
            > Duration::seconds(MAX_BLOCK_SECOND * 2)
        {
            if time::now_utc().to_timespec() - self.last_block_timestamp
                > Duration::seconds(MAX_BLOCK_SECOND * 6)
            {
                let mut new_peers = self.peers.clone();
                new_peers.remove(&self.blocker);

                let last_block = self.chain.lastest();
                let calculate_block = if last_block.is_some() {
                    vec![last_block.unwrap().clone()]
                } else {
                    vec![]
                };

                if let Some(pk) = calculate_leader::<P>(&new_peers, &calculate_block) {
                    return (true, Some(pk));
                }
            }
            return (true, None);
        }
        (false, None)
    }

    fn build_p2p_block(&mut self) -> (Vec<(P::PublicKey, Event<M, P>)>, Option<BlockID>) {
        if let Some(block) = self.create_block() {
            self.verify
                .insert(block.id().clone(), vec![self.pk.clone()]);
            self.blocks.insert(block.id().clone(), block.clone());
            (
                self.peers
                    .iter()
                    .map(|pk| {
                        (
                            pk.clone(),
                            Event::new_from_self_message(
                                self.pk.clone(),
                                Message::Block(block.clone()),
                                &self.psk,
                            ),
                        )
                    })
                    .collect(),
                Some(block.id().clone()),
            )
        } else {
            (vec![], None)
        }
    }

    fn build_p2p_verify(&mut self, block_id: &BlockID) -> Vec<(P::PublicKey, Event<M, P>)> {
        let event = Event::new_from_self_message(
            self.pk.clone(),
            Message::Verify(block_id.clone()),
            &self.psk,
        );
        self.peers
            .iter()
            .map(|pk| (pk.clone(), event.clone()))
            .collect()
    }

    fn build_p2p_commit(
        &mut self,
        block_id: &BlockID,
    ) -> (Vec<(P::PublicKey, Event<M, P>)>, Option<Block<M, P>>) {
        let peers_len = self.peers.len();
        let verify_len = self.verify.get(&block_id).unwrap_or(&vec![]).len();

        if (peers_len < 3 && peers_len == verify_len) || 2 * (peers_len + 1) <= 3 * verify_len {
            let pk = self.pk.clone();
            self.commit
                .entry(block_id.clone())
                .and_modify(|peers| peers.push(pk.clone()))
                .or_insert(vec![pk]);

            let event = Event::new_from_self_message(
                self.pk.clone(),
                Message::Commit(block_id.clone()),
                &self.psk,
            );
            (
                self.peers
                    .iter()
                    .map(|pk| (pk.clone(), event.clone()))
                    .collect(),
                self.confirm_block(block_id),
            )
        } else {
            (vec![], None)
        }
    }

    fn build_p2p_leader_expired(
        &mut self,
        new_pk: P::PublicKey,
    ) -> Vec<(P::PublicKey, Event<M, P>)> {
        // check if no leader
        if !self.leader_confirm.contains_key(&new_pk) {
            self.peers
                .iter()
                .map(|pk| {
                    (
                        pk.clone(),
                        Event::new_from_self_message(
                            self.pk.clone(),
                            Message::Leader(new_pk.clone(), self.chain.len() as u64),
                            &self.psk,
                        ),
                    )
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn build_p2p(
        &mut self,
        sender: &P::PublicKey,
        event: &Event<M, P>,
    ) -> Vec<(P::PublicKey, Event<M, P>)> {
        self.peers
            .iter()
            .filter_map(|pk| {
                if pk != &self.pk && pk != &event.creator && pk != sender {
                    Some((pk.clone(), event.clone()))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl<M: 'static + MessageTrait, P: 'static + Peer, A: QuickPBFTBridgeActor<M, P>> Actor
    for QuickPBFTActor<M, P, A>
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heart_beat(ctx);
    }
}

impl<M: 'static + MessageTrait, P: 'static + Peer, A: QuickPBFTBridgeActor<M, P>>
    Handler<HandleEventMessage<M, P>> for QuickPBFTActor<M, P, A>
{
    type Result = ();

    fn handle(&mut self, msg: HandleEventMessage<M, P>, ctx: &mut Self::Context) -> Self::Result {
        let (next_p2p, block) = self.handle_event(msg.0, msg.1, ctx);
        self.send_bridge(HandleEventResultMessage(next_p2p, block));
    }
}

impl<M: 'static + MessageTrait, P: 'static + Peer, A: QuickPBFTBridgeActor<M, P>>
    Handler<HandlePeerInMessage<P>> for QuickPBFTActor<M, P, A>
{
    type Result = ();

    fn handle(&mut self, msg: HandlePeerInMessage<P>, _ctx: &mut Self::Context) -> Self::Result {
        self.peers.insert(msg.0);
        // sync block
        let next_p2p = self.create_sync_request();
        self.send_bridge(HandleEventResultMessage(next_p2p, None));
    }
}

impl<M: 'static + MessageTrait, P: 'static + Peer, A: QuickPBFTBridgeActor<M, P>>
    Handler<HandlePeerOutMessage<P>> for QuickPBFTActor<M, P, A>
{
    type Result = ();

    fn handle(&mut self, msg: HandlePeerOutMessage<P>, _ctx: &mut Self::Context) -> Self::Result {
        self.peers.remove(&msg.0);
        if msg.0 == self.blocker {
            self.lead_blocker();
            println!("DEBUG: need re calculate leader: {}", self.blocker);
            let next_p2p = self.build_p2p_leader_expired(self.blocker.clone());
            self.send_bridge(HandleEventResultMessage(next_p2p, None));
        }
    }
}
