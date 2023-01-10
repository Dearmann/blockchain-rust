
pub mod block_miner;
pub mod cli;
pub mod mining_loop;
pub mod node_client;

fn main() {
    let args = cli::parse_args();
    let node_url = args.node_url.clone();
    let node_client = node_client::NetworkClient::new(node_url);

    mining_loop::mining_loop(args, node_client);
}
