use serde::{Deserialize, Serialize};

use super::{Address, Coin};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transaction {
    pub sender: Address,
    pub reciever: Address,
    pub amount: Coin,
}

impl Transaction {
    pub fn new(sender: Address, reciever: Address, amount: Coin) -> Self {
        Self {
            sender,
            reciever,
            amount,
        }
    }
}
