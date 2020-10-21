//! RandChain daemon client.

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate app_dirs;
extern crate env_logger;
extern crate libc;

extern crate chain;
extern crate db;
extern crate keys;
extern crate logs;
extern crate message;
extern crate network;
extern crate p2p;
extern crate primitives;
extern crate rpc as ethcore_rpc;
extern crate script;
extern crate storage;
extern crate sync;
extern crate verification;

mod commands;
mod config;
mod rpc;
mod rpc_apis;
mod seednodes;
mod util;

use app_dirs::AppInfo;

pub const APP_INFO: AppInfo = AppInfo {
    name: "randchaind",
    author: "RandChain",
};
pub const PROTOCOL_VERSION: u32 = 00_001;
pub const PROTOCOL_MINIMUM: u32 = 00_001;
pub const USER_AGENT: &'static str = "randchaind";
pub const REGTEST_USER_AGENT: &'static str = "/randchaind-testnet:0.0.1/";
pub const LOG_INFO: &'static str = "sync=info";

fn main() {
    // Always print backtrace on panic.
    ::std::env::set_var("RUST_BACKTRACE", "1");

    if let Err(err) = run() {
        println!("{}", err);
    }
}

fn run() -> Result<(), String> {
    let yaml = load_yaml!("cli.yml");
    let matches = clap::App::from_yaml(yaml).get_matches();
    let cfg = config::parse(&matches)?;

    if !cfg.quiet {
        if cfg!(windows) {
            logs::init(LOG_INFO, logs::DateLogFormatter);
        } else {
            logs::init(LOG_INFO, logs::DateAndColorLogFormatter);
        }
    } else {
        env_logger::init();
    }

    // TODO:
    // + import
    // + rollback

    commands::start(cfg)
}
