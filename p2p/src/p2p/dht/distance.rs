use serde_derive::{Deserialize, Serialize};
use std::cmp::Ordering;

/// DHT Distance use 160 bit (20bytes)
#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub(crate) struct Distance([u8; 20]);

impl Distance {
    pub fn distance(from: &[u8], base: &[u8]) -> Self {
        let from_len = from.len();
        let base_len = base.len();

        let (len, mut hold) = if from_len >= 20 && base_len >= 20 {
            (0, [0; 20])
        } else if from_len > base_len {
            let mut hold = [0; 20];
            hold[0..base_len].copy_from_slice(&from[base_len..]);
            (base_len, hold)
        } else {
            let mut hold = [0; 20];
            hold[0..base_len].copy_from_slice(&from[base_len..]);
            (from_len, hold)
        };

        hold[len..20].copy_from_slice(
            &((0..(20usize - len))
                .map(|i| from.get(i).unwrap() ^ base.get(i).unwrap())
                .collect::<Vec<u8>>()[..]),
        );
        Distance(hold)
    }
}

impl Default for Distance {
    fn default() -> Distance {
        let binary = Default::default();
        Distance(binary)
    }
}

impl Ord for Distance {
    fn cmp(&self, other: &Distance) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for Distance {
    fn partial_cmp(&self, other: &Distance) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Distance {
    fn eq(&self, other: &Distance) -> bool {
        self.0 == other.0
    }
}
