use anyhow::{anyhow, Result};
use clap::Parser;
use dirs::home_dir;
use std::{ffi::OsStr, fs, path::PathBuf, str::FromStr, time::Instant};
use subprocess::{Exec, Redirection};
use wol::{send_wol, MacAddr};

use wakenrun::{SshInstructions, Task, WakeupInstructions};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(value_name = "FILE")]
    config: PathBuf,
}

pub struct CmdResult {
    pub success: bool,
    pub std_out: String,
}

pub fn run_cmd(
    command: impl AsRef<OsStr>,
    args: &[impl AsRef<OsStr>],
    dir: Option<&PathBuf>,
    must_succeed: bool,
) -> Result<CmdResult> {
    let dir_ = match dir.as_ref() {
        Some(i) => i.as_os_str().to_owned(),
        None => home_dir().unwrap().into_os_string(),
    };

    // TODO: want to add option to tee the output so you can see live what it is doing
    let out = Exec::cmd(command)
        .cwd(dir_)
        .args(args)
        .stdout(Redirection::Pipe)
        .capture()?;
    let success = out.success();
    if must_succeed {
        assert!(success);
    }

    return Ok(CmdResult {
        success,
        std_out: out.stdout_str().trim().into(),
    });
}

fn wakeup(host: &String, i: WakeupInstructions, s: SshInstructions) -> Result<()> {
    if !i.enabled {
        return Ok(());
    }

    let mac_addr: MacAddr = MacAddr::from_str(&i.mac).unwrap();
    log::info!("Sending magic packet to {}", mac_addr);
    send_wol(mac_addr, None, None)?;

    let now = Instant::now();

    if i.validate_ping {
        log::info!("Waiting for host to respond to pings...");
        let args: Vec<String> = vec![
            format!("{}", host),
            String::from("-W"),
            String::from("2"),
            String::from("-c"),
            String::from("3"),
        ];
        let mut passed = false;

        while now.elapsed().as_secs() < i.boot_timeout_secs {
            let r = run_cmd(i.ping_cmd.as_str(), &args, None, false)?;

            if r.success {
                passed = true;
                break;
            }
        }

        if !passed {
            return Err(anyhow!("Host did not respond to ping before timeout"));
        }
    }

    if i.validate_ssh_connection {
        log::info!("Testing host SSH connection...");
        let mut args: Vec<String> = vec![];

        if s.ssh_identity_file.is_some() {
            args.push("-i".into());
            args.push(s.ssh_identity_file.unwrap());
            args.push("-o".into());
            args.push("IdentityOnly=yes".into());
        }

        if s.ssh_port.is_some() {
            args.push("-p".into());
            args.push(format!("{}", s.ssh_port.unwrap()));
        }

        match s.ssh_user {
            Some(v) => args.push(format!("{}@{}", v, &host)),
            None => args.push(host.to_string()),
        }
        args.push("whoami".into());

        let mut passed = false;
        while now.elapsed().as_secs() < i.boot_timeout_secs {
            let r = run_cmd(s.ssh_cmd.as_str(), &args, None, false)?;

            if r.success {
                log::info!("Remote whoami output: {}", r.std_out);
                passed = true;
                break;
            }
        }

        if !passed {
            return Err(anyhow!(
                "Host did not allow ssh connection with these args: {:#?}",
                args
            ));
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let config_fp = Cli::parse().config;

    if !config_fp.exists() {
        panic!("{:?} does not exist!", config_fp)
    }
    let data =
        fs::read_to_string(config_fp.clone()).expect(&format!("Unable to read {:?}", config_fp));
    let t: Task = serde_yaml::from_str(&data).expect("Unable to open config file");
    wakeup(&t.host, t.wakeup_instructions, t.ssh.clone())?;
    Ok(())
}
