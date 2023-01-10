use crate::{
    model::{
        Address, Block, BlockHash, Blockchain, Transaction, TransactionPool, TransactionVec,
        BLOCK_SUBSIDY,
    },
    util::{
        execution::{sleep_millis, Runnable},
        Context,
    },
};
use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MinerError {
    #[error("No valid block was mined at index `{0}`")]
    BlockNotMined(u64),
}

pub struct Miner {
    miner_address: Address,
    max_blocks: u64,
    max_nonce: u64,
    tx_waiting_ms: u64,
    blockchain: Blockchain,
    pool: TransactionPool,
    target: BlockHash,
}

impl Runnable for Miner {
    fn run(&self) -> Result<()> {
        self.start()
    }
}

impl Miner {
    pub fn new(context: &Context) -> Miner {
        let target = Self::create_target(context.config.difficulty);

        Miner {
            miner_address: context.config.miner_address.clone(),
            max_blocks: context.config.max_blocks,
            max_nonce: context.config.max_nonce,
            tx_waiting_ms: context.config.tx_waiting_ms,
            blockchain: context.blockchain.clone(),
            pool: context.pool.clone(),
            target,
        }
    }

    // Try to constanly calculate and append new valid blocks to the blockchain,
    // including all pending transactions in the transaction pool each time
    pub fn start(&self) -> Result<()> {
        info!(
            "start minining with difficulty {}",
            self.blockchain.difficulty
        );

        // In each loop it tries to find the next valid block and append it to the blockchain
        let mut block_counter = 0;
        loop {
            if self.must_stop_mining(block_counter) {
                info!("block limit reached, stopping mining");
                return Ok(());
            }

            // Empty all transactions from the pool, they will be included in the new block
            let transactions = self.pool.pop();

            // Do not try to mine a block if there are no transactions in the pool
            if transactions.is_empty() {
                sleep_millis(self.tx_waiting_ms);
                continue;
            }

            // try to find a valid next block of the blockchain
            let last_block = self.blockchain.get_last_block();
            let mining_result = self.mine_block(&last_block, &transactions.clone());
            match mining_result {
                Some(block) => {
                    info!("valid block found for index {}", block.index);
                    self.blockchain.add_block(block.clone())?;
                    block_counter += 1;
                }
                None => {
                    let index = last_block.index + 1;
                    error!("no valid block was foun for index {}", index);
                    return Err(MinerError::BlockNotMined(index).into());
                }
            }
        }
    }

    // Creates binary data mask with the amount of left padding zeroes indicated by the "difficulty" value
    // Used to easily compare if a newly created block has a hash that matches the difficulty
    fn create_target(difficulty: u32) -> BlockHash {
        BlockHash::MAX >> difficulty
    }

    // check if we have hit the limit of mined blocks (if the limit is set)
    fn must_stop_mining(&self, block_counter: u64) -> bool {
        self.max_blocks > 0 && block_counter >= self.max_blocks
    }

    // Tries to find the next valid block of the blockchain
    // It will create blocks with different "nonce" values until one has a hash that matches the difficulty
    // Returns either a valid block (that satisfies the difficulty) or "None" if no block was found
    fn mine_block(&self, last_block: &Block, transactions: &TransactionVec) -> Option<Block> {
        // Add the coinbase transaction as the first transaction in the block
        let coinbase = self.create_coinbase_transaction();
        let mut block_transactions = transactions.clone();
        block_transactions.insert(0, coinbase);

        for nonce in 0..self.max_nonce {
            let next_block = self.create_next_block(last_block, block_transactions.clone(), nonce);

            // A valid block must have a hash with enough starting zeroes
            // To check that, we simply compare against a binary data mask
            if next_block.hash < self.target {
                return Some(next_block);
            }
        }

        None
    }

    // Creates a valid next block for a blockchain
    // Takes into account the index and the hash of the previous block
    fn create_next_block(
        &self,
        last_block: &Block,
        transactions: TransactionVec,
        nonce: u64,
    ) -> Block {
        let index = (last_block.index + 1) as u64;
        let previous_hash = last_block.hash;

        // hash of the new block is automatically calculated on creation
        Block::new(index, nonce, previous_hash, transactions)
    }

    fn create_coinbase_transaction(&self) -> Transaction {
        Transaction {
            sender: Address::default(),
            recipient: self.miner_address.clone(),
            amount: BLOCK_SUBSIDY,
        }
    }
}
