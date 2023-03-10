use std::{thread, time};
use spec::{
    types::{Address, Transaction},
    validators::BLOCK_SUBSIDY,
};

use crate::{block_miner::mine_block, cli::MinerArgs, node_client::NodeClient};

pub fn run_mining_loop(args: MinerArgs, node_client: impl NodeClient) {
    let mut blocks_mined: u64 = 0;

    while should_keep_mining(blocks_mined, &args) {
        // The block template already includes the correct index, previous_hash and transactions for the next valid block
        let mut block_template = node_client.get_block_template();

        //if there are no transaction - dont mine new block
        if block_template.transactions.is_empty(){
            thread::sleep(time::Duration::from_secs(10));
            continue;
        }

        // Add the coinbase transaction as the first transaction in the block
        let coinbase = create_coinbase_transaction(args.miner_address.clone());
        block_template.transactions.insert(0, coinbase);
        block_template.hash = block_template.calculate_hash();

        // Try to mine the new block
        let mining_result = mine_block(&args, &block_template);
        match mining_result {
            Some(new_block) => {
                println!("Block mined");
                node_client.submit_block(&new_block);
                blocks_mined += 1;
            }
            None => {
                println!("Error mining block");
            }
        }
    }
}

pub fn create_coinbase_transaction(miner_address: Address) -> Transaction {
    Transaction {
        sender: Address::default(),
        recipient: miner_address,
        amount: BLOCK_SUBSIDY,
    }
}

fn should_keep_mining(blocks_mined: u64, args: &MinerArgs) -> bool {
    if args.max_blocks == 0 {
        return true;
    }
    blocks_mined < args.max_blocks
}
