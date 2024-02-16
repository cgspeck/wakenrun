use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::{fs, path::PathBuf, str::FromStr};
use wakenrun::{SshInstructions, Task, WakeupInstructions};
use wol::{send_wol, MacAddr};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(value_name = "FILE")]
    config: PathBuf,
}

fn wakeup(i: WakeupInstructions, s: SshInstructions) -> Result<()> {
    if !i.enabled {
        return Ok(());
    }

    let mac_addr: MacAddr = MacAddr::from_str(&i.mac).unwrap();
    println!("Sending magic packet to {}", mac_addr);
    send_wol(mac_addr, None, None)?;

    if i.validate_ping {
        // TODO: ping loop until timeout
    }

    if i.validate_ssh_connection {
        // TODO: attempt ssh connection
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let config_fp = Cli::parse().config;

    if !config_fp.exists() {
        panic!("{:?} does not exist!", config_fp)
    }
    let data =
        fs::read_to_string(config_fp.clone()).expect(&format!("Unable to read {:?}", config_fp));
    let task: Task = serde_yaml::from_str(&data).expect("Unable to open config file");
    wakeup(task.wakeup_instructions, task.ssh.clone())?;
    Ok(())
}
