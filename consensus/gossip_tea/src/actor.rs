use std::collections::HashMap;

use teatree::actor::prelude::*;
use teatree::primitives::types::EventID;
use teatree::traits::propose::Peer;

use crate::message::{Gossip, GossipConfirm, GossipMessage, GossipNew, GossipP2P, GossipPeerLeave};

const K_NUM: usize = 1; // set how many peer when select to gossip one time;

pub type See<P> = HashMap<<P as Peer>::PublicKey, <P as Peer>::Signature>;
pub type Sees<P> = HashMap<<P as Peer>::PublicKey, See<P>>;
pub type SeeMap<P> = HashMap<<P as Peer>::PublicKey, Sees<P>>;

pub struct GossipActor<P: 'static + Peer> {
    pk: P::PublicKey,
    psk: P::PrivateKey,
    k_num: usize,
    recipient_gossip_p2p: Recipient<GossipP2P<P>>,
    recipient_gossip_confirm: Recipient<GossipConfirm<P>>,
    rate_numerator: usize,
    rate_denominator: usize,
    see_map: HashMap<EventID, SeeMap<P>>,
}

impl<P: 'static + Peer> GossipActor<P> {
    pub fn new(
        pk: P::PublicKey,
        psk: P::PrivateKey,
        recipient_gossip_p2p: Recipient<GossipP2P<P>>,
        recipient_gossip_confirm: Recipient<GossipConfirm<P>>,
    ) -> Self {
        let see_map = HashMap::new();
        let rate_numerator: usize = 2;
        let rate_denominator: usize = 3;
        let k_num = K_NUM;

        Self {
            pk,
            psk,
            k_num,
            recipient_gossip_p2p,
            recipient_gossip_confirm,
            rate_numerator,
            rate_denominator,
            see_map,
        }
    }

    pub fn new_with_k_num(
        pk: P::PublicKey,
        psk: P::PrivateKey,
        k_num: usize,
        recipient_gossip_p2p: Recipient<GossipP2P<P>>,
        recipient_gossip_confirm: Recipient<GossipConfirm<P>>,
    ) -> Self {
        let see_map = HashMap::new();
        let rate_numerator: usize = 2;
        let rate_denominator: usize = 3;

        Self {
            pk,
            psk,
            k_num,
            recipient_gossip_p2p,
            recipient_gossip_confirm,
            rate_numerator,
            rate_denominator,
            see_map,
        }
    }

    pub fn new_with_rate(
        pk: P::PublicKey,
        psk: P::PrivateKey,
        k_num: usize,
        recipient_gossip_p2p: Recipient<GossipP2P<P>>,
        recipient_gossip_confirm: Recipient<GossipConfirm<P>>,
        rate_numerator: usize,
        rate_denominator: usize,
    ) -> Self {
        let see_map = HashMap::new();

        Self {
            pk,
            psk,
            k_num,
            recipient_gossip_p2p,
            recipient_gossip_confirm,
            rate_numerator,
            rate_denominator,
            see_map,
        }
    }

    pub fn confirm(&self, event_id: &EventID) -> bool {
        if self
            .see_map
            .get(event_id)
            .and_then(|event_sees| {
                event_sees.get(&self.pk).and_then(|sees| {
                    let num = sees.len();

                    for (_, sees) in sees.iter() {
                        if sees.len() * self.rate_denominator < num * self.rate_numerator {
                            return None;
                        }
                    }

                    return Some(());
                })
            })
            .is_some()
        {
            true
        } else {
            false
        }
    }

    pub fn slowest(
        &self,
        event_id: &EventID,
        k_num: usize,
        from: &P::PublicKey,
    ) -> Vec<(P::PublicKey, SeeMap<P>)> {
        self.see_map
            .get(event_id)
            .and_then(|event_sees| {
                let mut slowest_names = vec![];
                let mut slowest_see_num = 0;
                let see_min_num =
                    event_sees.len() * self.rate_numerator / self.rate_denominator + 1;

                for (name, sees) in event_sees.iter() {
                    if name == &self.pk {
                        continue;
                    }

                    let mut num = 0;
                    for (_, see) in sees.iter() {
                        if see.len() < see_min_num {
                            num += 1;
                        }
                    }

                    if num > slowest_see_num {
                        if slowest_names.len() <= k_num {
                            slowest_names.push(name);
                        } else {
                            slowest_names.pop();
                            slowest_names.push(name);
                        }
                        slowest_see_num = num;
                    } else if num == slowest_see_num {
                        if slowest_names.contains(&from) {
                            slowest_names.remove_item(&from);
                            slowest_names.push(name);
                        }
                    }
                }

                Some(
                    slowest_names
                        .iter()
                        .map(|slowest_name| ((*slowest_name).clone(), event_sees.clone()))
                        .collect(),
                )
            })
            .unwrap_or(vec![])
    }
}

impl<P: 'static + Peer> Actor for GossipActor<P> {
    type Context = Context<Self>;
}

impl<P: 'static + Peer> Handler<GossipNew<P>> for GossipActor<P> {
    type Result = ();

    fn handle(&mut self, msg: GossipNew<P>, _ctx: &mut Self::Context) -> Self::Result {
        let (event_id, mut peers) = (msg.0, msg.1);
        if self.see_map.contains_key(&event_id) {
            return;
        }

        if !peers.contains(&self.pk) {
            peers.push(self.pk.clone());
        }

        let mut sees = HashMap::new();
        for pk in peers.iter() {
            sees.insert(pk.clone(), HashMap::new());
        }

        let mut event_sees = HashMap::new();
        while !peers.is_empty() {
            let pk = peers.pop().unwrap();
            event_sees.insert(pk, sees.clone());
        }

        let sign = P::sign(&self.psk, &event_id.to_vec());
        let pk = self.pk.clone();

        event_sees.get_mut(&pk).and_then(|sees| {
            sees.get_mut(&pk)
                .and_then(|see| Some(see.insert(pk.clone(), sign)))
        });

        self.see_map.insert(event_id.clone(), event_sees);

        let mut nexts = self.slowest(&event_id, self.k_num, &self.pk);
        while !nexts.is_empty() {
            let (name, see) = nexts.pop().unwrap();
            let _ = self.recipient_gossip_p2p.do_send(GossipP2P(
                self.pk.clone(),
                name,
                GossipMessage(self.pk.clone(), Gossip(event_id.clone(), see)),
            ));
        }
    }
}

impl<P: 'static + Peer> Handler<GossipPeerLeave<P>> for GossipActor<P> {
    type Result = ();

    fn handle(&mut self, msg: GossipPeerLeave<P>, _ctx: &mut Self::Context) -> Self::Result {
        let pk = msg.0;
        let mut confirmed_events: Vec<EventID> = vec![];

        for (_, sees) in self.see_map.iter_mut() {
            if sees.contains_key(&pk) {
                sees.remove(&pk);
                let _ = sees.iter_mut().map(|(_, see)| {
                    see.remove(&pk);
                });
            }
        }

        for event_id in self.see_map.keys() {
            if self.confirm(event_id) {
                confirmed_events.push(event_id.clone());
            }
        }

        while !confirmed_events.is_empty() {
            let event_id = confirmed_events.pop().unwrap();
            let see_map = self.see_map.remove(&event_id).unwrap();
            let _ = self.recipient_gossip_confirm.do_send(GossipConfirm(
                self.pk.clone(),
                event_id,
                see_map,
            ));
        }
    }
}

impl<P: 'static + Peer> Handler<GossipMessage<P>> for GossipActor<P> {
    type Result = ();

    fn handle(&mut self, msg: GossipMessage<P>, _ctx: &mut Self::Context) -> Self::Result {
        let (from, event_id, mut from_event_sees) = (msg.0, (msg.1).0, (msg.1).1);
        let event_bytes = event_id.to_vec();

        let self_pk = &self.pk;
        self.see_map.get_mut(&event_id).and_then(|event_sees| {
            let mut my_self_sees = event_sees.get(self_pk).unwrap().clone();

            for (pk, mut sees) in from_event_sees.drain() {
                if &pk == self_pk {
                    continue;
                }

                event_sees.get_mut(&pk).and_then(|self_sees| {
                    for (pk, mut see) in sees.drain() {
                        self_sees
                            .entry(pk.clone())
                            .and_modify(|self_see| {
                                for (see_pk, see_sign) in see.drain() {
                                    // TODO check sign
                                    if P::verify(&see_pk, &event_bytes, &see_sign) {
                                        my_self_sees.get_mut(&pk).and_then(|my_self_see| {
                                            Some(
                                                my_self_see
                                                    .entry(see_pk.clone())
                                                    .or_insert(see_sign.clone()),
                                            )
                                        });
                                        my_self_sees.get_mut(self_pk).and_then(|my_self_see| {
                                            Some(
                                                my_self_see
                                                    .entry(see_pk.clone())
                                                    .or_insert(see_sign.clone()),
                                            )
                                        });

                                        self_see.entry(see_pk).or_insert(see_sign);
                                    }
                                }
                            })
                            .or_insert(see);
                    }
                    Some(())
                });
            }

            event_sees
                .get_mut(self_pk)
                .and_then(|self_sees| Some(*self_sees = my_self_sees));

            Some(())
        });

        let mut nexts = self.slowest(&event_id, self.k_num, &from);

        while !nexts.is_empty() {
            let (name, see) = nexts.pop().unwrap();
            let _ = self.recipient_gossip_p2p.do_send(GossipP2P(
                self.pk.clone(),
                name,
                GossipMessage(self.pk.clone(), Gossip(event_id.clone(), see)),
            ));
        }

        if self.confirm(&event_id) {
            let see_map = self.see_map.remove(&event_id).unwrap();
            let _ = self.recipient_gossip_confirm.do_send(GossipConfirm(
                self.pk.clone(),
                event_id,
                see_map,
            ));
        }
    }
}
