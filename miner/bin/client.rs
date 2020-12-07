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
struct GenkeyOpts {}

/// A subcommand for ....
#[derive(Clap)]
struct MineOpts {}

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
