use serde_json::json;

use teatree::actor::prelude::*;
use teatree::crypto::keypair::{PrivateKey, PublicKey};
use teatree::primitives::types::{GroupID, PeerAddr};
use teatree::traits::message::bridge_message::{EventMessage, LocalResponseMessage};

use teatree::NetworkBridgeActor;

use crate::actor::DSActor;
use crate::event::Event;
use crate::message::{ReadResult, ID};

pub(crate) struct ReadCloseMessage(pub ID);

impl Message for ReadCloseMessage {
    type Result = ();
}

pub(crate) struct ReadResultMessage(pub Vec<u8>);

impl Message for ReadResultMessage {
    type Result = ();
}

pub(crate) struct AppendTaskMessage(pub usize, pub Option<Recipient<ReadResult>>);

impl Message for AppendTaskMessage {
    type Result = ();
}

struct PreRead {
    group_id: GroupID,
    self_pk: PublicKey,
    self_psk: PrivateKey,
    peer_addr: PeerAddr,
    id: ID,
}

impl PreRead {
    fn genereate(&self) -> EventMessage {
        EventMessage(
            self.group_id.clone(),
            self.peer_addr.clone(),
            bincode::serialize(&Event::new_read(
                self.id.clone(),
                &self.self_pk,
                &self.self_psk,
            ))
            .unwrap_or(vec![]),
        )
    }
}

pub(crate) struct ReadActor {
    read: PreRead,
    bridge: Addr<NetworkBridgeActor>,
    ds_addr: Addr<DSActor>,
    tasks: Vec<(usize, Option<Recipient<ReadResult>>)>,
}

impl ReadActor {
    pub fn new(
        group_id: GroupID,
        peer_addr: PeerAddr,
        self_pk: PublicKey,
        self_psk: PrivateKey,
        id: ID,
        bridge: Addr<NetworkBridgeActor>,
        ds_addr: Addr<DSActor>,
        receiver_index: usize,
        receiver_addr: Option<Recipient<ReadResult>>,
    ) -> Self {
        let pre_read = PreRead {
            group_id,
            self_pk,
            self_psk,
            peer_addr,
            id,
        };

        ReadActor {
            read: pre_read,
            bridge: bridge,
            ds_addr: ds_addr,
            tasks: vec![(receiver_index, receiver_addr)],
        }
    }
}

impl Actor for ReadActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        self.bridge.do_send(self.read.genereate());
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        self.ds_addr.do_send(ReadCloseMessage(self.read.id.clone()));

        Running::Stop
    }
}

impl Handler<ReadResultMessage> for ReadActor {
    type Result = ();

    fn handle(&mut self, msg: ReadResultMessage, _ctx: &mut Self::Context) -> Self::Result {
        let data = msg.0;

        if self.tasks.len() == 1 {
            let (index, addr) = self.tasks.pop().unwrap();
            if addr.is_some() {
                let _ = addr.unwrap().do_send(ReadResult(index, Some(data)));
            } else {
                self.bridge.do_send(LocalResponseMessage(
                    self.read.group_id.clone(),
                    index,
                    Some(json!({
                        "data": String::from_utf8(data).unwrap_or(Default::default())
                    })),
                ))
            }
        } else {
            while !self.tasks.is_empty() {
                let (index, addr) = self.tasks.pop().unwrap();
                if addr.is_some() {
                    let _ = addr.unwrap().do_send(ReadResult(index, Some(data.clone())));
                } else {
                    self.bridge.do_send(LocalResponseMessage(
                        self.read.group_id.clone(),
                        index,
                        Some(json!({
                            "data": String::from_utf8(data.clone()).unwrap_or(Default::default())
                        })),
                    ))
                }
            }
        }
    }
}

impl Handler<AppendTaskMessage> for ReadActor {
    type Result = ();

    fn handle(&mut self, msg: AppendTaskMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.tasks.push((msg.0, msg.1));
    }
}
