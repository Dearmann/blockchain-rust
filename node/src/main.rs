#[macro_use]
extern crate log;

mod api;
mod database;
mod peer;
mod server;
mod util;

use env_logger::{Builder, Target};
use log::LevelFilter;
use crate::util::config::parse_from_cli;
use crate::server::Server;

fn main() {
    // set up logger
    setup_logger();
    info!("starting up");

    // read configuration from cli
    let config = parse_from_cli();

    // run server
    let server = Server::new(config);
    server.start();

    // terminate program on ctrl + c
    initialize_terminate_handler();
}

fn setup_logger() {
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.filter(None, LevelFilter::Info);
    builder.init();
}

pub fn initialize_terminate_handler() {
    ctrlc::set_handler(move || {
        std::process::exit(0);
    })
    .expect("Error setting terminator");
}
