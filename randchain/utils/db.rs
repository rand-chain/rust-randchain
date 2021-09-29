use super::config::Config;
use app_dirs::{app_dir, AppDataType};
use crypto::sr25519::PK;
use db;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::Arc;
use {storage, APP_INFO};

fn custom_path(data_dir: &str, sub_dir: &str) -> PathBuf {
    let mut path = PathBuf::from(data_dir);
    path.push(sub_dir);
    create_dir_all(&path).expect("Failed to get app dir");
    path
}

pub fn open_db(data_dir: Option<String>, db_cache: usize) -> storage::SharedStore {
    let db_path = match data_dir {
        Some(data_dir_str) => custom_path(&data_dir_str, "db"),
        None => app_dir(AppDataType::UserData, &APP_INFO, "db").expect("Failed to get app dir"),
    };
    Arc::new(
        db::BlockChainDatabase::open_at_path(db_path, db_cache).expect("Failed to open database"),
    )
}

pub fn init_db(cfg: &Config) -> Result<(), String> {
    // insert genesis block if db is empty
    let genesis_block = cfg.network.genesis_block();
    match cfg.db.block_hash(0) {
        Some(ref db_genesis_block_hash) if db_genesis_block_hash != genesis_block.hash() => {
            Err("Trying to open database with incompatible genesis block".into())
        }
        Some(_) => Ok(()),
        None => {
            let hash = genesis_block.hash().clone();
            cfg.db
                .insert(genesis_block)
                .expect("Failed to insert genesis block to the database");
            cfg.db
                .canonize(&hash)
                .expect("Failed to canonize genesis block");
            Ok(())
        }
    }
}

pub fn create_node_table(data_dir: Option<String>) -> PathBuf {
    let mut node_table = match data_dir {
        Some(s) => custom_path(&s, "p2p"),
        None => app_dir(AppDataType::UserData, &APP_INFO, "p2p").expect("Failed to get app dir"),
    };
    node_table.push("nodes.csv");
    node_table
}

pub fn create_account_dir(data_dir: Option<String>) -> PathBuf {
    let account_dir_pathbuf = match data_dir {
        Some(s) => custom_path(&s, "account"),
        None => {
            app_dir(AppDataType::UserData, &APP_INFO, "account").expect("Failed to get app dir")
        }
    };
    account_dir_pathbuf
}