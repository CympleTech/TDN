use std::slice::Iter;

use crate::block::BlockID;

const CAPACITY: usize = 100;

pub(crate) struct FixedChain(Vec<BlockID>);

impl FixedChain {
    pub fn new(vec: Vec<BlockID>) -> Self {
        FixedChain(vec)
    }

    pub fn vec(&self) -> &Vec<BlockID> {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> Iter<BlockID> {
        self.0.iter()
    }

    pub fn contains(&mut self, block_id: &BlockID) -> bool {
        self.0.contains(block_id)
    }

    pub fn lastest(&self) -> Option<&BlockID> {
        if self.0.len() < 1 {
            return None;
        }

        self.0.get(self.0.len() - 1)
    }

    pub fn push(&mut self, block_id: BlockID) {
        if self.0.contains(&block_id) {
            return;
        }

        if self.0.len() < CAPACITY {
            self.0.push(block_id);
        } else {
            self.0.remove(0);
            self.0.push(block_id);
        }
    }
}
