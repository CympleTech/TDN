use std::cmp::Ordering;
use std::collections::BTreeSet;
use teatree::crypto::hash::H256;
use teatree::traits::propose::Peer;

use crate::block::BlockID;

pub fn calculate_leader<P: Peer>(
    peers: &BTreeSet<P::PublicKey>,
    blocks: &Vec<BlockID>,
) -> Option<P::PublicKey> {
    if peers.is_empty() {
        return None;
    }

    let refer = H256::new(&bincode::serialize(blocks).unwrap()[..]);
    let mut distances: Vec<(Distance, &P::PublicKey)> = peers
        .iter()
        .map(|pk| {
            (
                Distance::new(&refer, &H256::new(&bincode::serialize(pk).unwrap()[..])),
                pk,
            )
        })
        .collect();

    distances.sort_by(|(d, _), pre| d.cmp(&pre.0));
    let (_, pk) = distances.remove(0);

    Some(pk.clone())
}

#[derive(Eq, Debug)]
struct Distance(Vec<u8>);

impl Distance {
    fn new(refer: &H256, origin: &H256) -> Self {
        let new_origin = origin.to_vec();
        Distance(
            refer
                .to_vec()
                .iter()
                .enumerate()
                .map(|(i, e)| ((*e as i16 - *(new_origin.get(i).unwrap()) as i16).abs()) as u8)
                .collect(),
        )
    }
}

impl PartialOrd for Distance {
    fn partial_cmp(&self, other: &Distance) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Distance {
    fn cmp(&self, other: &Distance) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialEq for Distance {
    fn eq(&self, other: &Distance) -> bool {
        self.0 == other.0
    }
}
