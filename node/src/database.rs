mod accounts;
mod blocks;
mod mempool;

use std::sync::{Arc, RwLock, RwLockReadGuard};

use anyhow::Result;
use spec::types::{Address, Block, Coin, Network, Transaction};
use spec::validators::{validate_block, validate_transaction};
use spec::Database as SpecDatabase;

use self::accounts::AccountDatabase;
use self::blocks::BlockDatabase;
use self::mempool::Mempool;

// by only exporting this struct, the caller does not need to worry about concurrency implementation
// as concurrent atomic writes from different threads are supported
#[derive(Clone)]
pub struct ConcurrentNodeDatabase(Arc<RwLock<NodeDatabase>>);

impl ConcurrentNodeDatabase {
    pub fn new(network: Network) -> Self {
        let database = NodeDatabase::new(network);
        let arc_rwlock_database = Arc::new(RwLock::new(database));

        Self(arc_rwlock_database)
    }

    pub fn append_block(&self, block: &Block) -> Result<()> {
        self.0.write().unwrap().append_block(block)
    }

    pub fn add_transaction(&self, transaction: Transaction) -> Result<()> {
        self.0.write().unwrap().add_transaction(transaction)
    }

    fn get_read_lock(&self) -> RwLockReadGuard<NodeDatabase> {
        self.0.read().unwrap()
    }
}

impl SpecDatabase for ConcurrentNodeDatabase {
    fn get_network(&self) -> Network {
        self.get_read_lock().get_network()
    }

    fn get_all_blocks(&self) -> Vec<Block> {
        self.get_read_lock().get_all_blocks()
    }

    fn get_tip_block(&self) -> Option<Block> {
        self.get_read_lock().get_tip_block()
    }

    fn get_account_balance(&self, address: &Address) -> Option<Coin> {
        self.get_read_lock().get_account_balance(address)
    }

    fn get_mempool_transactions(&self) -> Vec<Transaction> {
        self.get_read_lock().get_mempool_transactions()
    }
}

// The non-concurrent implementation of the database is not exported
#[derive(Clone)]
struct NodeDatabase {
    network: Network,
    block_db: BlockDatabase,
    account_db: AccountDatabase,
    mempool: Mempool,
}

impl NodeDatabase {
    pub fn new(network: Network) -> Self {
        Self {
            network,
            block_db: BlockDatabase::default(),
            account_db: AccountDatabase::default(),
            mempool: Mempool::default(),
        }
    }

    pub fn append_block(&mut self, block: &Block) -> Result<()> {
        validate_block(self, block)?;

        // add new block to blockchain
        self.block_db.append_block(block.clone());

        // update balances
        Self::process_transactions(&mut self.account_db, block)?;

        // remove submitted transactions
        self.mempool.remove_transactions(&block.transactions);

        Ok(())
    }

    fn process_transactions(account_db: &mut AccountDatabase, block: &Block) -> Result<()> {

        let mut transactions = block.transactions.iter();
        let coinbase = transactions.next().unwrap();
        account_db.add_funds(&coinbase.reciever, coinbase.amount);
        for transaction in transactions {
            account_db.transfer(&transaction.sender, &transaction.reciever, transaction.amount)?;
        }

        Ok(())
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<()> {
        validate_transaction(self, &transaction)?;
        self.mempool.add_transaction(transaction);

        Ok(())
    }
}

//necessary for spec validators
impl SpecDatabase for NodeDatabase {
    fn get_network(&self) -> Network {
        self.network.clone()
    }

    fn get_all_blocks(&self) -> Vec<Block> {
        self.block_db.get_all_blocks()
    }

    fn get_tip_block(&self) -> Option<Block> {
        self.block_db.get_tip_block()
    }

    fn get_account_balance(&self, address: &Address) -> Option<Coin> {
        self.account_db.get_account_balance(address)
    }

    fn get_mempool_transactions(&self) -> Vec<Transaction> {
        self.mempool.get_transactions()
    }
}
