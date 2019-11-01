use serde_derive::{Deserialize, Serialize};
use std::collections::hash_map::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use crate::crypto::keypair::PublicKey;

use super::binary_tree::Node;
//use crate::storage::append_node_list;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct DHTTable {
    cells: Node,
    #[serde(skip)]
    need_hb: HashMap<PublicKey, SocketAddr>,
    #[serde(skip)]
    heartbeating: HashMap<PublicKey, Instant>,
    #[serde(skip)]
    tmp_cells: HashMap<PublicKey, Option<SocketAddr>>,
}

impl DHTTable {
    pub fn new(pk: &PublicKey) -> Self {
        DHTTable {
            cells: Node::root(pk),
            need_hb: HashMap::new(),
            heartbeating: HashMap::new(),
            tmp_cells: HashMap::new(),
        }
    }

    pub fn next_hb_peers(&mut self) -> (Vec<(PublicKey, SocketAddr)>, Vec<PublicKey>) {
        // update heartingbeating
        let mut dis: Vec<PublicKey> = Vec::new();

        let need_remove: Vec<PublicKey> = self
            .heartbeating
            .iter()
            .filter(|(_, ins)| Instant::now().duration_since(**ins) > Duration::new(20, 0))
            .map(|(pk, _)| pk)
            .cloned()
            .collect();

        for pk in need_remove {
            self.heartbeating.remove(&pk);
            self.cells.remove(&pk);
            dis.push(pk);
        }

        if self.need_hb.len() == 0 {
            self.need_hb = self
                .cells
                .all()
                .iter()
                .map(|(pk, socket)| ((*pk).clone(), (*socket).clone()))
                .collect();
        }

        let key = self.need_hb.keys().into_iter().last();
        if key.is_none() {
            return (vec![], dis);
        }

        let hb_pk = key.unwrap().clone();
        if let Some(socket_addr) = self.need_hb.remove(&hb_pk) {
            if self.heartbeating.get(&hb_pk).is_some() {
                (vec![], dis)
            } else {
                self.heartbeating.insert(hb_pk.clone(), Instant::now());
                (vec![(hb_pk, socket_addr)], dis)
            }
        } else {
            (vec![], dis)
        }
    }

    /// update peers liveness when receive heartboat, return disconnect peer
    pub fn update_hb_peers(&mut self, pk: &PublicKey) {
        self.heartbeating.remove(pk);
        //self.cells.get_mut(&id.pk).and_then(|c| Some(c.liveness(1)));
    }

    /// peer leave or remove
    pub fn remove_peer(&mut self, pk: &PublicKey) {
        self.need_hb.remove(pk);
        self.heartbeating.remove(pk);
        self.cells.remove(pk);
        self.tmp_cells.remove(pk);
    }

    /// peer join return is_new bool
    pub fn add_peer(&mut self, pk: &PublicKey, socket_addr: SocketAddr) -> bool {
        self.cells.insert(self.cells.new(pk, socket_addr))
    }

    /// when peer first join, remeber it's peer_id, and socket_addr
    pub fn add_tmp_peer(&mut self, pk: &PublicKey, socket_addr: Option<SocketAddr>) -> bool {
        // if had in cells, return true
        if self.cells.contains(pk) {
            return true;
        }

        if let Some(cell) = self.tmp_cells.get_mut(pk) {
            *cell = socket_addr;
            false
        } else {
            self.tmp_cells.insert(pk.clone(), socket_addr);
            true
        }
    }

    pub fn fixed_peer(&mut self, pk: &PublicKey) -> bool {
        if self.cells.contains(pk) {
            return false;
        }

        if let Some(sock) = self.tmp_cells.remove(pk) {
            if sock.is_some() {
                return self.add_peer(pk, sock.unwrap());
            }
            //append_node_list(self.cells.pk(), vec![(pk.clone(), sock)]);
        }
        return false;
    }

    pub fn fixed_tmp_peer(&mut self, pk: &PublicKey, socket_addr: SocketAddr) {
        self.tmp_cells
            .get_mut(pk)
            .map(|socket| *socket = Some(socket_addr));
    }

    /// get peer's socket addr by public key
    pub fn get_socket_addr(&self, pk: &PublicKey) -> Option<SocketAddr> {
        match self.cells.search(pk) {
            Some((socket, true)) => Some(socket),
            Some((_, false)) | None => match self.tmp_cells.get(&pk) {
                Some(cell) => *cell,
                None => None,
            },
        }
    }

    pub fn check_add(&self, pk: &PublicKey) -> bool {
        !self.cells.contains(pk)
    }

    pub fn contains(&self, pk: &PublicKey) -> bool {
        self.cells.contains(pk)
    }

    /// Help Peer Build DHT
    pub fn _dht_help(&self, _pk: &PublicKey) -> Vec<(PublicKey, SocketAddr)> {
        // TODO
        self.cells
            .all()
            .iter()
            .map(|(pk, socket)| ((*pk).clone(), (*socket).clone()))
            .collect()
    }
}
