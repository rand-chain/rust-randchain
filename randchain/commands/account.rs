use clap::ArgMatches;
use hex;
use raccount::keypair::KeyPair;
use raccount::password::{password_from_file, password_prompt};
use rand;
use std::fs;
use std::path::PathBuf;
use utils;

fn get_account_dir(matches: &clap::ArgMatches) -> PathBuf {
    let data_dir = match matches.value_of("data-dir") {
        Some(s) => Some(s.to_owned()),
        None => None,
    };
    utils::create_account_dir(data_dir)
}

fn new_cmd(matches: &ArgMatches) -> Result<(), String> {
    let seed: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
    let (sk, pk) = crypto::sr25519::create_keypair(seed.as_slice());
    let keypair = KeyPair { sk, pk };

    let name: String = match matches.value_of("name") {
        Some(s) => s.to_owned(),
        None => hex::encode(pk.to_bytes()),
    };
    let mut path = get_account_dir(matches);
    path.push(name);
    keypair.save(&path)
}

fn list_cmd(matches: &ArgMatches) -> Result<(), String> {
    let path = get_account_dir(matches);
    let names = fs::read_dir(path).unwrap();
    for n in names {
        println!("Name: {}", n.unwrap().path().display())
    }

    Ok(())
}

pub fn start(matches: &ArgMatches) -> Result<(), String> {
    let data_dir = match matches.value_of("data-dir") {
        Some(s) => Some(s.to_owned()),
        None => None,
    };
    utils::create_account_dir(data_dir);

    let r = match matches.subcommand() {
        ("new", Some(new_matches)) => new_cmd(new_matches),
        ("list", Some(list_matches)) => list_cmd(list_matches),
        _ => unreachable!(),
    };
    r
}
