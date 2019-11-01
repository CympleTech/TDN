use std::collections::HashMap;
use std::mem::transmute;
use std::time::Duration;

use teatree::actor::prelude::*;
use teatree::crypto::keypair::PublicKey;
use teatree::primitives::types::EventID;
use teatree::traits::propose::Peer;
use teatree::traits::sample::peer::NetworkPeer;
use teatree::{system_init, system_run};

use gossip_tea::*;

struct TotalActor {
    addrs: HashMap<PublicKey, Addr<GossipActor<NetworkPeer>>>,
    confirmed: HashMap<EventID, HashMap<PublicKey, Gossip<NetworkPeer>>>,
    times: HashMap<EventID, usize>,
}

impl TotalActor {
    fn start_round(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_later(Duration::new(1, 0), |act, _ctx| {
            let peers: Vec<PublicKey> = act.addrs.iter().map(|(pk, _)| pk).cloned().collect();
            for i in 0u32..2u32 {
                let bytes: [u8; 4] = unsafe { transmute(i.to_be()) };
                let event_id = EventID::new(&bytes);
                act.confirmed.insert(event_id.clone(), HashMap::new());
                act.times.insert(event_id.clone(), 0);

                for (pk, addr) in act.addrs.iter() {
                    println!("{} start...", pk);
                    addr.do_send(GossipNew(event_id.clone(), peers.clone()));
                }
            }
        });
    }
}

impl Actor for TotalActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        for _ in 0..4 {
            let (pk, psk) = NetworkPeer::generate();

            let new_addr = ctx.address();
            let new_pk = pk.clone();

            let gossip_addr = GossipActor::create(move |ctx| {
                ctx.set_mailbox_capacity(100);
                GossipActor::new(
                    pk,
                    psk,
                    new_addr.clone().recipient::<GossipP2P<NetworkPeer>>(),
                    new_addr.recipient::<GossipConfirm<NetworkPeer>>(),
                )
            });

            self.addrs.insert(new_pk, gossip_addr);
        }

        self.start_round(ctx);
    }
}

impl Handler<GossipP2P<NetworkPeer>> for TotalActor {
    type Result = ();

    fn handle(&mut self, msg: GossipP2P<NetworkPeer>, _ctx: &mut Self::Context) -> Self::Result {
        let (from, to, event_id) = (&msg.0, &msg.1, msg.event_id());
        self.times.get_mut(&event_id).and_then(|t| {
            *t += 1;
            Some(println!("times: {}", t))
        });

        if self
            .confirmed
            .get(event_id)
            .and_then(|c| {
                c.get(&to).and_then(|gossip| {
                    let confirmed_gossip = (*gossip).clone();
                    self.addrs.get(from).and_then(move |a| {
                        Some(a.do_send(GossipMessage(from.clone(), confirmed_gossip)))
                    })
                })
            })
            .is_none()
        {
            self.addrs.get(to).and_then(|a| Some(a.do_send(msg.2)));
        }
    }
}

impl Handler<GossipConfirm<NetworkPeer>> for TotalActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: GossipConfirm<NetworkPeer>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let (pk, event_id, sees) = (msg.0, msg.1, msg.2);
        println!("confirmed: {}", pk);
        self.confirmed
            .get_mut(&event_id)
            .and_then(|pks| Some(pks.entry(pk).or_insert(Gossip(event_id, sees))));

        for (_, confirms) in self.confirmed.iter() {
            if confirms.len() == self.addrs.len() {
                System::current().stop();
            }
        }
    }
}

fn main() {
    let runner = system_init();

    TotalActor::create(|ctx| {
        ctx.set_mailbox_capacity(100);
        TotalActor {
            addrs: HashMap::new(),
            confirmed: HashMap::new(),
            times: HashMap::new(),
        }
    });

    println!("Hello, Tea!, We will start!");
    system_run(runner);
}
