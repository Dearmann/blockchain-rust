use spec::types::Transaction;

// Represents a pool of unrealized transactions
#[derive(Debug, Clone, Default)]
pub struct Mempool {
    transactions: Vec<Transaction>,
}

impl Mempool {
    pub fn get_transactions(&self) -> Vec<Transaction> {
        self.transactions.clone()
    }

    // Add a new transaction to the pool
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
        info!("transaction added");
    }

    pub fn remove_transactions(&mut self, transactions: &[Transaction]) {
        // TODO: transactions should have a nonce to avoid duplicates
        self.transactions.retain(|t| !transactions.contains(t));
    }
}
