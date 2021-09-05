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
extern crate logs;
extern crate message;
extern crate network;
extern crate p2p;
extern crate primitives;
extern crate rpc as ethcore_rpc;
extern crate storage;
extern crate sync;
extern crate verification;

mod commands;
mod config;
mod utils;

use app_dirs::AppInfo;

pub const APP_INFO: AppInfo = AppInfo {
    name: "randchain",
    author: "RandChain",
};
pub const PROTOCOL_VERSION: u32 = 70_014;
pub const PROTOCOL_MINIMUM: u32 = 70_001;
pub const USER_AGENT: &'static str = "/Satoshi:0.12.1/";
pub const REGTEST_USER_AGENT: &'static str = "randchain-regtest";
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
    let app = clap::App::from_yaml(yaml).setting(clap::AppSettings::ArgRequiredElseHelp);
    let matches = app.get_matches();
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

    match matches.subcommand() {
        // ("import", Some(import_matches)) => commands::import(cfg, import_matches),
        // ("rollback", Some(rollback_matches)) => commands::rollback(cfg, rollback_matches),
        ("node", Some(_node_matches)) => commands::node::start(cfg, _node_matches),
        _ => Err("Please specify a subcommand".to_owned()),
    }
}