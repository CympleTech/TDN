use serde_derive::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::iter::Iterator;
use std::net::SocketAddr;

//use primitives::consts::K_BUCKET;
use crate::crypto::keypair::PublicKey;
use crate::primitives::consts::P2P_DEFAULT_SOCKET;

use super::distance::Distance;

type TreeNode = Option<Box<Node>>;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Node {
    left: TreeNode,
    right: TreeNode,
    pk: PublicKey,
    distance: Distance,
    value: SocketAddr,
}

impl Node {
    pub fn root(base: &PublicKey) -> Self {
        Node {
            left: None,
            right: None,
            pk: base.clone(),
            distance: Distance::distance(&base.to_bytes(), &base.to_bytes()),
            value: P2P_DEFAULT_SOCKET.parse().unwrap(),
        }
    }

    pub fn new(&self, pk: &PublicKey, value: SocketAddr) -> Self {
        Node {
            left: None,
            right: None,
            pk: pk.clone(),
            distance: Distance::distance(&pk.to_bytes(), &self.pk.to_bytes()),
            value: value,
        }
    }

    /// return if is_new and successsful
    pub fn insert(&mut self, node: Node) -> bool {
        if self.distance < node.distance {
            if let Some(ref mut right) = self.right {
                if right.pk == node.pk {
                    if right.value == node.value {
                        return false;
                    }
                    right.value = node.value;
                } else {
                    return right.insert(node);
                }
            } else {
                self.right = Some(Box::new(node));
            }
        } else {
            if let Some(ref mut left) = self.left {
                if left.pk == node.pk {
                    if left.value == node.value {
                        return false;
                    }
                    left.value = node.value;
                } else {
                    return left.insert(node);
                }
            } else {
                self.left = Some(Box::new(node));
            }
        }
        true
    }

    /// search the pk's socketaddr or pk's nearest socketaddr
    pub fn search(&self, pk: &PublicKey) -> Option<(SocketAddr, bool)> {
        if &self.pk == pk {
            return Some((self.value.clone(), true));
        }

        if let Some(ref left) = self.left {
            let next = left.search(pk);
            if next.is_some() {
                return next;
            }
        }

        if let Some(ref right) = self.right {
            let next = right.search(pk);
            if next.is_some() {
                return next;
            }
        }

        None
    }

    pub fn remove(&mut self, pk: &PublicKey) {
        if let Some(ref mut left) = self.left {
            if &left.pk == pk {
                self.left = None;
                return;
            }
            left.remove(pk);
        }

        if let Some(ref mut right) = self.right {
            if &right.pk == pk {
                self.right = None;
                return;
            }

            right.remove(pk);
        }
    }

    pub fn contains(&self, pk: &PublicKey) -> bool {
        if let Some((_, true)) = self.search(pk) {
            true
        } else {
            false
        }
    }

    //pub fn iter(&self) -> impl Iterator<Item = (&PublicKey, &SocketAddr)> {}

    fn collect_element<'a>(&'a self, vec: &mut Vec<(&'a PublicKey, &'a SocketAddr)>) {
        if let Some(ref left) = self.left {
            vec.push((&left.pk, &left.value));
            left.collect_element(vec);
        }

        if let Some(ref right) = self.right {
            vec.push((&right.pk, &right.value));
            right.collect_element(vec);
        }
    }

    pub fn all(&self) -> Vec<(&PublicKey, &SocketAddr)> {
        let mut vec = vec![];
        self.collect_element(&mut vec);
        vec
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Node) -> Ordering {
        self.distance.cmp(&other.distance)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Node) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Node {}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.pk == other.pk
    }
}

impl Iterator for Node {
    type Item = (PublicKey, SocketAddr);

    fn next(&mut self) -> Option<Self::Item> {
        Some((Default::default(), self.value))
    }
}
