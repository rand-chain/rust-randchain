use clap::ArgMatches;
use std::path::PathBuf;
use utils;
use utils::password::{password_from_file, password_prompt};

#[derive(Debug, PartialEq)]
pub struct ListAccounts {
    pub path: Option<String>,
}

impl ListAccounts {
    pub fn new(_matches: &clap::ArgMatches) -> Self {
        Self { path: None }
    }
}

#[derive(Debug, PartialEq)]
pub struct NewAccount {
    pub iterations: u32,
    pub path: Option<String>,
    pub password_file: Option<String>,
}

impl NewAccount {
    pub fn new(matches: &clap::ArgMatches) -> Self {
        let iterations: u32 = matches
            .value_of("key-iterations")
            .unwrap_or("0")
            .parse()
            .unwrap();
        let password_file = matches.value_of("password").map(|x| x.to_string());
        Self {
            iterations,
            path: None,
            password_file,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ImportAccounts {
    pub from: Vec<String>,
    pub to: String,
}

impl ImportAccounts {
    pub fn new(matches: &clap::ArgMatches) -> Self {
        let data_dir = match matches.value_of("data-dir") {
            Some(s) => Some(s.to_owned()),
            None => None,
        };
        let data_dir_pathbuf = utils::create_account_dir(data_dir);
        let data_dir_owned = data_dir_pathbuf.to_str().unwrap().to_owned();

        let from: Vec<_> = matches
            .values_of("import-path")
            .expect("CLI argument is required; qed")
            .map(|s| s.to_string())
            .collect();
        Self {
            from,
            to: data_dir_owned,
        }
    }
}

fn new_cmd(matches: &ArgMatches) -> Result<(), String> {
    let cmd = NewAccount::new(matches);
    let password = match cmd.password_file {
        Some(file) => password_from_file(file)?,
        None => password_prompt()?,
    };

    let acc_provider = account_provider(
        cmd.path,
        Some(cmd.iterations), /* sstore_iterations */
        None,                 /* refresh_time */
    )?;

    let new_account = acc_provider
        .new_account(&password)
        .map_err(|e| format!("Could not create new account: {}", e))?;

    // Ok(format!("0x{:x}", new_account))
    Ok(())
}

fn list_cmd(matches: &ArgMatches) -> Result<(), String> {
    let cmd = ListAccounts::new(matches);
    let acc_provider = account_provider(
        cmd.path, None, /* sstore_iterations */
        None, /* refresh_time */
    )?;

    let accounts = acc_provider.accounts().map_err(|e| format!("{}", e))?;
    let result = accounts
        .into_iter()
        .map(|a| format!("0x{:x}", a))
        .collect::<Vec<String>>()
        .join("\n");

    Ok(())
    // Ok(result)
}

fn import_cmd(matches: &ArgMatches) -> Result<(), String> {
    let cmd = ImportAccounts::new(matches);
    let to = keys_dir(cmd.to)?;
    let mut imported = 0;

    for path in &cmd.from {
        let path = PathBuf::from(path);
        if path.is_dir() {
            let from = RootDiskDirectory::at(&path);
            imported += import_accounts(&from, &to)
                .map_err(|e| format!("Importing accounts from {:?} failed: {}", path, e))?
                .len();
        } else if path.is_file() {
            import_account(&path, &to)
                .map_err(|e| format!("Importing account from {:?} failed: {}", path, e))?;
            imported += 1;
        }
    }

    Ok(())
    // Ok(format!("{} account(s) imported", imported))
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
        ("import", Some(import_matches)) => import_cmd(import_matches),
        _ => unreachable!(),
    };
    r
}
