use std::{thread, time};
use spec::{
    types::{Address, Transaction},
    validators::BLOCK_SUBSIDY,
};

use crate::{block_miner::mine_block, cli::MinerArgs, node_client::NodeClient};

pub fn run_mining_loop(args: MinerArgs, node_client: impl NodeClient) {
    let mut blocks_mined_by_miner: u64 = 0;

    // while should_keep_mining(blocks_mined_by_miner, &args) {
    loop {
        // Client has template
        let mut template_block = node_client.get_block_template();

        //if there are no transaction - dont mine new block
        if template_block.transactions.is_empty() && blocks_mined_by_miner != 0 {
                thread::sleep(time::Duration::from_secs(10));
                continue;
        }

        // Add the reward transaction as the first transaction in the block
        let reward = create_coinbase_transaction(args.miner_address.clone());
        template_block.transactions.insert(0, reward);
        template_block.hash = template_block.calculate_hash();

        // Try to mine the new block
        let mining_result = mine_block(&args, &template_block);
        match mining_result {
            Some(new_block) => {
                blocks_mined_by_miner += 1;
                println!("Block number {} mined", blocks_mined_by_miner);
                node_client.submit_block(&new_block);
            }
            None => {
                println!("Error while mining block");
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
