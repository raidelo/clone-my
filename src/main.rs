use std::io::{Read, Write};

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

const CONFIG_FOLDER: &str = "clone-my";
const CONFIG_FILE: &str = "config.toml";

fn main() -> Result<()> {
    let cmd = clap::Command::new("clone-my")
        .subcommands([
            clap::Command::new("set-account").args([clap::Arg::new("account")
                .help("main account")
                .required(true)]),
        ])
        .args([clap::Arg::new("repo").help("repository to clone")])
        .arg_required_else_help(true);

    let args = cmd.get_matches();

    let Some(home_path) = home::home_dir() else {
        bail!("unable to get the home directory")
    };

    let config_file = home_path
        .join(".config")
        .join(CONFIG_FOLDER)
        .join(CONFIG_FILE);

    match args.subcommand() {
        Some(("set-account", sub_m)) => {
            let account = sub_m
                .get_one::<String>("account")
                .ok_or(anyhow!("account argument must be provided"))?;

            let url = format!("https://github.com/{}", account);
            let config = Config::new(url);

            std::fs::create_dir_all(config_file.parent().unwrap())?;

            let mut f = std::fs::File::create(config_file)?;

            write!(f, "{}", toml::ser::to_string_pretty(&config)?)?;
        }
        Some(_) => bail!("unknown options provided"),
        None => {
            let repo = args
                .get_one::<String>("repo")
                .ok_or(anyhow!("repo argument must be provided"))?;

            let mut f = std::fs::File::open(&config_file).map_err(|_| {
                anyhow!("no hay cuenta configurada, ejecuta primero: clone-my set-account <cuenta>")
            })?;

            let mut buf = String::new();
            f.read_to_string(&mut buf)?;

            let config = toml::from_str::<Config>(&buf)?;

            clone_repo(&format!("{}/{}.git", config.main_account, repo))?;
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    main_account: String,
}

impl Config {
    pub fn new(account: String) -> Self {
        Self {
            main_account: account,
        }
    }
}

pub fn clone_repo(repo_url: &str) -> Result<()> {
    let status = std::process::Command::new("git")
        .args(["clone", repo_url])
        .spawn()?
        .wait()?;
    if !status.success() {
        bail!("git clone falló con código: {:?}", status.code());
    }
    Ok(())
}
