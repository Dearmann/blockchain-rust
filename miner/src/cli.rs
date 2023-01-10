use clap::Parser;
use spec::types::Address;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct MinerArgs {
    /// Miner address
    #[clap(short = 'a', long, value_parser)]
    pub miner_address: Address,

    /// Node address
    #[clap(short = 'n', long, value_parser,  default_value = "http://localhost:8000")]
    pub node_url: String,

    /// Difficulty
    #[clap(short = 'd', long, value_parser, default_value = "10")]
    pub difficulty: u32,

    /// Maximum nonce
    #[clap(long, value_parser, default_value = "1000000")]
    pub max_nonce: u64,
}

pub fn parse_args() -> MinerArgs {
    MinerArgs::parse()
}
