extern crate clap;

use clap::Clap;

/// RandChain miner client
#[derive(Clap)]
struct Opts {
    #[clap(subcommand)]
    command: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    /// Generate a key pair
    GenKey(GenkeyOpts),
    /// Connect to randchaind rpc port and mine with a key
    Mine(MineOpts),
}

/// A subcommand for ....
#[derive(Clap)]
struct GenkeyOpts {
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
        SubCommand::GenKey(o) => {
            gen_key(o);
        }
        SubCommand::Mine(o) => {
            mine(o);
        }
    }
}

fn gen_key(opts: GenkeyOpts) {
    unimplemented!();
}

fn mine(opts: MineOpts) {
    unimplemented!();
}
