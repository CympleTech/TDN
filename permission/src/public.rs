use serde_derive::{Deserialize, Serialize};
use std::collections::hash_map::Iter;
use std::collections::HashMap;

use teatree::actor::prelude::Addr;
use teatree::primitives::functions::get_default_storage_path;
use teatree::primitives::types::{GroupID, PeerAddr};
use teatree::storage::{DiskDatabase, DiskStorageActor, Entity, EntityWrite};
use teatree::traits::propose::{Group as GroupTrait, Peer};

#[derive(Clone)]
pub struct PublicGroup<P: Peer> {
    id: GroupID,
    pk: P::PublicKey,
    peer_addr: PeerAddr,
    peers: HashMap<P::PublicKey, PeerAddr>,
    storage: Addr<DiskStorageActor>,
}

impl<P: 'static + Peer> PublicGroup<P> {
    pub fn load(id: GroupID, pk: P::PublicKey, peer_addr: PeerAddr) -> Self {
        let mut path = get_default_storage_path();
        path.push("group");
        path.push(format!("{}", id));
        let db = DiskDatabase::new(Some(path.clone()));

        let peers = if let Ok(group) = db.read_entity::<GroupStore<P>>(id.to_string()) {
            group.1
        } else {
            HashMap::new()
        };

        drop(db);

        let storage = DiskStorageActor::run(Some(path));

        Self {
            id,
            pk,
            peer_addr,
            peers,
            storage,
        }
    }

    pub fn has_peer(&self, pk: &P::PublicKey) -> bool {
        self.peers.contains_key(pk)
    }

    pub fn get_peer_addr(&self, pk: &P::PublicKey) -> Option<PeerAddr> {
        self.peers.get(pk).cloned()
    }

    pub fn get_by_peer_addr(&self, peer_addr: &PeerAddr) -> Option<&P::PublicKey> {
        self.peers
            .iter()
            .filter_map(|(pk, addr)| if addr == peer_addr { Some(pk) } else { None })
            .next()
    }

    pub fn all_peer_keys(&self) -> Vec<P::PublicKey> {
        self.peers.keys().map(|e| e).cloned().collect()
    }

    pub fn iter(&self) -> Iter<P::PublicKey, PeerAddr> {
        self.peers.iter()
    }
}

impl<P: 'static + Peer> GroupTrait<P> for PublicGroup<P> {
    type JoinType = (P::PublicKey, P::Signature);

    fn id(&self) -> &GroupID {
        &self.id
    }

    fn join(&mut self, data: Self::JoinType, peer_addr: PeerAddr) -> bool {
        if self.has_peer(&data.0) {
            return true;
        }

        let (pk, sign) = (data.0, data.1);
        if P::verify(&pk, &peer_addr.to_vec(), &sign) {
            // TODO DHT
            self.peers.insert(pk.clone(), peer_addr);
            self.storage.do_send(EntityWrite(GroupStore::<P>(
                self.id.clone(),
                self.peers.clone(),
            )));
            true
        } else {
            false
        }
    }

    fn leave(&mut self, peer_addr: &PeerAddr) -> bool {
        let pk = self
            .peers
            .iter()
            .filter_map(|(pk, addr)| {
                if addr == peer_addr {
                    Some(pk.clone())
                } else {
                    None
                }
            })
            .next();

        if pk.is_some() {
            self.peers.remove(&pk.unwrap());
        }

        true
    }

    fn verify(&self, pk: &P::PublicKey) -> bool {
        self.peers.contains_key(pk)
    }

    fn help_sync_peers(&self, _pk: &P::PublicKey) -> Vec<PeerAddr> {
        self.peers
            .iter()
            .map(|(_pk, peer_addr)| peer_addr)
            .cloned()
            .collect()
    }

    fn add_sync_peers(&mut self, pk: &P::PublicKey, peer_addr: PeerAddr) {
        self.peers.entry(pk.clone()).or_insert(peer_addr);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GroupStore<P: Peer>(GroupID, HashMap<P::PublicKey, PeerAddr>);

impl<P: Peer> Entity for GroupStore<P> {
    type Key = String;

    fn key(&self) -> Self::Key {
        self.0.to_string()
    }
}
