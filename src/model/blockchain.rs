use anyhow::Result;
use std::{
    slice::Iter,
    sync::{Arc, Mutex},
};
use thiserror::Error;

use super::{account_balance_map::AccountBalanceMap, Block, BlockHash, Transaction};

pub type BlockVec = Vec<Block>;

// We don't need to export this because concurrency is encapsulated in this file
type SyncedBlockVec = Arc<Mutex<BlockVec>>;
type SyncedAccountBalanceVec = Arc<Mutex<AccountBalanceMap>>;

pub const BLOCK_SUBSIDY: u64 = 100;

// Error types to return when trying to add blocks with invalid fields
#[derive(Error, PartialEq, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum BlockchainError {
    #[error("Invalid index")]
    InvalidIndex,

    #[error("Invalid previous_hash")]
    InvalidPreviousHash,

    #[error("Invalid hash")]
    InvalidHash,

    #[error("Invalid difficulty")]
    InvalidDifficulty,

    #[error("Coinbase transaction not found")]
    CoinbaseTransactionNotFound,

    #[error("Invalid coinbase amount")]
    InvalidCoinbaseAmount,
}

// Struct that holds all the blocks in the blockhain
// Multiple threads can read/write concurrently to the list of blocks
#[derive(Debug, Clone)]
pub struct Blockchain {
    pub difficulty: u32,
    blocks: SyncedBlockVec,
    account_balances: SyncedAccountBalanceVec,
}

// Basic operations in the blockchain are encapsulated in the implementation
// Encapsulates concurrency concerns, so external callers do not need to know how it's handled
impl Blockchain {
    // Creates a brand new blockchain with a genesis block
    pub fn new(difficulty: u32) -> Blockchain {
        let genesis_block = Blockchain::create_genesis_block();

        // add the genesis block to the synced vec of blocks
        let blocks = vec![genesis_block];
        let synced_blocks = Arc::new(Mutex::new(blocks));
        let synced_account_balances = SyncedAccountBalanceVec::default();

        Blockchain {
            difficulty,
            blocks: synced_blocks,
            account_balances: synced_account_balances,
        }
    }

    fn create_genesis_block() -> Block {
        let index = 0;
        let nonce = 0;
        let previous_hash = BlockHash::default();
        let transactions = Vec::new();

        let mut block = Block::new(index, nonce, previous_hash, transactions);

        // to easily sync multiple nodes in a network, the genesis blocks must match
        // so we clear the timestamp so the hash of the genesis block is predictable
        block.timestamp = 0;
        block.hash = block.calculate_hash();

        block
    }

    // Returns a copy of the most recent block in the blockchain
    pub fn get_last_block(&self) -> Block {
        let blocks = self.blocks.lock().unwrap();

        blocks[blocks.len() - 1].clone()
    }

    // Returns a copy of the whole list of blocks
    pub fn get_all_blocks(&self) -> BlockVec {
        let blocks = self.blocks.lock().unwrap();

        blocks.clone()
    }

    // Tries to append a new block into the blockchain
    // It will validate that the values of the new block are consistend with the blockchain state
    // This operation is safe to be called concurrently from multiple threads
    pub fn add_block(&self, block: Block) -> Result<()> {
        // the "blocks" attribute is protected by a Mutex
        // so only one thread at a time can access the value when the lock is held
        // that prevents adding multiple valid blocks at the same time
        // preserving the correct order of indexes and hashes of the blockchain
        let mut blocks = self.blocks.lock().unwrap();
        let last = &blocks[blocks.len() - 1];

        // check that the index is valid
        if block.index != last.index + 1 {
            return Err(BlockchainError::InvalidIndex.into());
        }

        // check that the previous_hash is valid
        if block.previous_hash != last.hash {
            return Err(BlockchainError::InvalidPreviousHash.into());
        }

        // check that the hash matches the data
        if block.hash != block.calculate_hash() {
            return Err(BlockchainError::InvalidHash.into());
        }

        // check that the difficulty is correct
        if block.hash.leading_zeros() < self.difficulty {
            return Err(BlockchainError::InvalidDifficulty.into());
        }

        // update the account balances by processing the block transactions
        self.update_account_balances(&block.transactions)?;

        // append the block to the end
        blocks.push(block);

        Ok(())
    }

    fn update_account_balances(&self, transactions: &[Transaction]) -> Result<()> {
        let mut account_balances = self.account_balances.lock().unwrap();
        // note that if any transaction (including coinbase) is invalid, an error will be returned before updating the balances
        let new_account_balances =
            Blockchain::calculate_new_account_balances(&account_balances, transactions)?;
        *account_balances = new_account_balances;

        Ok(())
    }

    fn calculate_new_account_balances(
        account_balances: &AccountBalanceMap,
        transactions: &[Transaction],
    ) -> Result<AccountBalanceMap> {
        // we work on a copy of the account balances
        let mut new_account_balances = account_balances.clone();
        let mut iter = transactions.iter();

        // the first transaction is always the coinbase transaction
        // in which the miner receives the mining rewards
        Blockchain::process_coinbase(&mut new_account_balances, iter.next())?;

        // the rest of the transactions are regular transfers between accounts
        Blockchain::process_transfers(&mut new_account_balances, iter)?;

        Ok(new_account_balances)
    }

    fn process_coinbase(
        account_balances: &mut AccountBalanceMap,
        coinbase: Option<&Transaction>,
    ) -> Result<()> {
        // The coinbase transaction is required in a valid block
        let coinbase = match coinbase {
            Some(transaction) => transaction,
            None => return Err(BlockchainError::CoinbaseTransactionNotFound.into()),
        };

        // In coinbase transactions, we only need to check that the amount is valid,
        // because whoever provides a valid proof-of-work block can receive the new coins
        let is_valid_amount = coinbase.amount == BLOCK_SUBSIDY;
        if !is_valid_amount {
            return Err(BlockchainError::InvalidCoinbaseAmount.into());
        }

        // The amount is valid so we add the new coins to the miner's address
        account_balances.add_amount(&coinbase.recipient, coinbase.amount);

        Ok(())
    }

    fn process_transfers(
        new_account_balances: &mut AccountBalanceMap,
        transaction_iter: Iter<Transaction>,
    ) -> Result<()> {
        // each transaction is validated using the updated account balances from previous transactions
        // that means that we allow multiple transacions from the same address in the same block
        // as long as they are consistent
        for tx in transaction_iter {
            new_account_balances.transfer(&tx.sender, &tx.recipient, tx.amount)?
        }

        Ok(())
    }
}
