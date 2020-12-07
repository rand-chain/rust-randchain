extern crate clap;
extern crate ecvrf;

use clap::Clap;
use ecvrf::{VrfPk, VrfSk};

/// RandChain miner client
#[derive(Clap)]
struct Opts {
    #[clap(subcommand)]
    command: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    /// Generate a key pair
    KeyGen(KeyGenOpts),
    /// Connect to randchaind rpc port and mine with a key
    Mine(MineOpts),
}

/// A subcommand for ....
#[derive(Clap)]
struct KeyGenOpts {
    /// Output public key file
    #[clap(short = "u", long = "pub", default_value = "pub.key")]
    pubkey: String,
    /// Output private key file
    #[clap(short = "r", long = "pri", default_value = "pri.key")]
    prikey: String,
}

/// A subcommand for ....
#[derive(Clap)]
struct MineOpts {
    /// Output public key file
    #[clap(short = "u", long = "pub", default_value = "pub.key")]
    pubkey: String,
    /// randchaind rpc port
    #[clap(short = "p", long = "port", default_value = "8333")]
    port: u16,
}

fn main() {
    let opts: Opts = Opts::parse();
    match opts.command {
        SubCommand::KeyGen(o) => {
            key_gen(o);
        }
        SubCommand::Mine(o) => {
            mine(o);
        }
    }
}

fn key_gen(opts: KeyGenOpts) {
    if std::path::Path::new(&opts.prikey).exists() {
        println!("{} existed", &opts.prikey);
        return;
    }
    if std::path::Path::new(&opts.pubkey).exists() {
        println!("{} existed", &opts.pubkey);
        return;
    }

    let (sk, pk) = ecvrf::keygen();
    std::fs::write(&opts.prikey, sk.to_bytes()).expect("save prikey error");
    println!("PriKey saved to: {}", opts.prikey);
    std::fs::write(&opts.pubkey, pk.to_bytes()).expect("save pubkey error");
    println!("PubKey saved to: {}", opts.pubkey);
}

fn mine(opts: MineOpts) {
    unimplemented!();
}
