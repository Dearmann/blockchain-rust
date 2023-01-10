use chrono::prelude::*;
use serde::{Deserialize, Serialize};

use crate::Database;

use super::{
    hash::{ConsensusHash, ConsensusHashable},
    Transaction,
};


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub nonce: u64,
    pub previous_hash: ConsensusHash,
    pub hash: ConsensusHash,
    pub transactions: Vec<Transaction>,
}

impl Block {
    // Create new block with default hash
    pub fn new(
        index: u64,
        nonce: u64,
        previous_hash: ConsensusHash,
        transactions: Vec<Transaction>,
    ) -> Block {
        let mut block = Block {
            index,
            timestamp: Utc::now().timestamp_millis(),
            nonce,
            previous_hash,
            hash: ConsensusHash::default(),
            transactions,
        };
        block.hash = block.calculate_hash();

        block
    }

    pub fn new_template<T: Database>(database: &T) -> Block {
        let (index, previous_hash) = match database.get_tip_block() {
            Some(tip_block) => (tip_block.index + 1, tip_block.hash),
            None => {
                // Template for genesis block
                let index = 0;
                let previous_hash = database.get_network().consensus_hash();
                (index, previous_hash)
            }
        };

        let transactions = database.get_mempool_transactions();

        Block::new(index, 0, previous_hash, transactions)
    }

    // Calculate hash of block
    pub fn calculate_hash(&self) -> ConsensusHash {
        let mut hashable_data = self.clone();
        hashable_data.hash = ConsensusHash::default();

        hashable_data.consensus_hash()
    }
}
