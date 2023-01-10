use miner::{cli, mining_loop::mining_loop, node_client::NetworkClient};

fn main() {
    let args = cli::parse_args();
    let node_url = args.node_url.clone();
    let node_client = NetworkClient::new(node_url);

    mining_loop(args, node_client);
}
