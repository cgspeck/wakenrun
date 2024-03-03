use anyhow::{anyhow, Result};
use clap::Parser;
use dirs::home_dir;
use log::{error, info};
use std::{
    ffi::OsStr, fs, path::PathBuf, process::exit, str::FromStr, thread::sleep, time::{Duration, Instant}
};
use subprocess::{Exec, Redirection};
use wol::{send_wol, MacAddr};

use wakenrun::{
    generate_sample_config, ShutdownInstructions, SshInstructions, Task, WakeupInstructions,
};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Config file to execute
    #[arg(value_name = "FILE")]
    config: PathBuf,
    /// Write a sample config to FILE
    #[arg(short, long, default_value = "false")]
    generate: bool,
}

pub struct CmdResult {
    pub success: bool,
    pub std_err: String,
    pub std_out: String,
}

pub fn run_cmd(
    command: impl AsRef<OsStr> + std::fmt::Display,
    args: &[impl AsRef<OsStr> + std::fmt::Display],
    dir: Option<&PathBuf>,
    must_succeed: bool,
    tee: bool,
) -> Result<CmdResult> {
    let dir_ = match dir.as_ref() {
        Some(i) => i.as_os_str().to_owned(),
        None => home_dir().unwrap().into_os_string(),
    };

    let mut command_ = command.to_string();
    let mut args_: Vec<String> = args.into_iter().map(|e| e.to_string()).collect();

    if command.to_string().split(" ").count() > 1 {
        for (i, w) in command.to_string().split(" ").enumerate() {
            if i == 0 {
                command_ = w.to_string()
            } else {
                args_.insert(i - 1, w.to_string())
            }
        }
    }

    if tee {
        let mut display_str = String::new();
        display_str += format!("{:#?}", dir_)
            .strip_prefix('"')
            .and_then(|x| x.strip_suffix('"'))
            .expect(format!("Unable to parse dir_: {:#?}", dir_).as_str());
        display_str += format!("$ {}", command_).as_str();
        for a in args_.clone() {
            display_str += " ";
            display_str += format!("{:?}", a)
                .strip_prefix('"')
                .and_then(|x| x.strip_suffix('"'))
                .expect(format!("Unable to parse arg: {:#?}", a).as_str());
        }

        info!("Running: {}", display_str);
    }

    let mut p = Exec::cmd(command_)
        .cwd(dir_)
        .args(&args_)
        .stdout(Redirection::Pipe)
        .popen()?;

    let mut std_out = String::new();
    let mut std_err = String::new();

    loop {
        let r = p.communicate(None);
        let m = r.unwrap();
        match m.0 {
            Some(v) => {
                if v.len() > 0 {
                    std_out += &v;
                    if tee {
                        info!("{}", v)
                    }
                }
            }
            None => (),
        }

        match m.1 {
            Some(v) => {
                if v.len() > 0 {
                    std_err += &v;
                    if tee {
                        error!("{}", v)
                    }
                }
            }
            None => (),
        }
        p.poll();
        if p.exit_status().is_some() {
            break;
        }
    }

    let success = p.exit_status().unwrap().success();
    if must_succeed {
        assert!(success);
    }

    return Ok(CmdResult {
        success,
        std_out: std_out,
        std_err: std_err,
    });
}

fn run_remote_cmd(host: &String, s: SshInstructions, remote_command: String) -> Result<CmdResult> {
    let mut args: Vec<String> = vec![];
    args.push("-t".into());

    if s.ssh_identity_file.is_some() {
        args.push("-i".into());
        args.push(s.ssh_identity_file.unwrap());
        args.push("-o".into());
        args.push("IdentitiesOnly=yes".into());
    }

    if s.ssh_port.is_some() {
        args.push("-p".into());
        args.push(format!("{}", s.ssh_port.unwrap()));
    }

    match s.ssh_user {
        Some(v) => args.push(format!("{}@{}", v, &host)),
        None => args.push(host.to_string()),
    }
    args.push(remote_command);
    run_cmd(s.ssh_cmd.as_str(), &args, None, true, true)
}

fn wakeup(
    host: &String,
    ping_cmd: &String,
    sleep_duration: &Duration,
    i: WakeupInstructions,
    s: SshInstructions,
) -> Result<()> {
    if !i.enabled {
        return Ok(());
    }

    let mac_addr: MacAddr = MacAddr::from_str(&i.mac).unwrap();
    info!("Sending magic packet to {}", mac_addr);
    send_wol(mac_addr, None, None)?;

    let now = Instant::now();

    if i.validate_ping {
        info!("Waiting for host to respond to pings...");
        let args: Vec<String> = vec![
            format!("{}", host),
            String::from("-W"),
            String::from("2"),
            String::from("-c"),
            String::from("3"),
        ];
        let mut passed = false;

        while now.elapsed().as_secs() < i.boot_timeout_secs {
            let r = run_cmd(ping_cmd.as_str(), &args, None, false, true)?;

            if r.success {
                passed = true;
                break;
            }

            sleep(*sleep_duration);
        }

        if !passed {
            return Err(anyhow!("Host did not respond to ping before timeout"));
        }
    }

    if i.validate_ssh_connection {
        log::info!("Testing host SSH connection...");

        let mut passed = false;
        while now.elapsed().as_secs() < i.boot_timeout_secs {
            let r = run_remote_cmd(&host, s.clone(), "whoami".into())?;
            if r.success {
                log::info!("Remote whoami output: {}", r.std_out);
                passed = true;
                break;
            }
        }

        if !passed {
            return Err(anyhow!("Host did not allow ssh connection."));
        }
    }

    Ok(())
}

fn shutdown(
    host: &String,
    ping_cmd: &String,
    sleep_duration: &Duration,
    i: ShutdownInstructions,
    s: SshInstructions,
) -> Result<()> {
    if !i.shutdown_remote {
        return Ok(());
    }

    info!("Shutting down remote...");
    run_remote_cmd(&host, s.clone(), i.shutdown_cmd)?;

    if i.validate_shutdown {
        let now = Instant::now();
        info!("Waiting for host to stop responding to pings...");
        let args: Vec<String> = vec![
            format!("{}", host),
            String::from("-W"),
            String::from("2"),
            String::from("-c"),
            String::from("3"),
        ];
        let mut passed = false;

        while now.elapsed().as_secs() < i.shutdown_timeout_secs {
            let r = run_cmd(ping_cmd.as_str(), &args, None, false, true)?;

            if !r.success {
                passed = true;
                break;
            }

            sleep(*sleep_duration);
        }

        if !passed {
            return Err(anyhow!("Host did appear to go offline before timeout"));
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let args = Cli::parse();
    let config_fp = args.config;

    if args.generate {
        return generate_sample_config(&config_fp);
    }

    if !config_fp.exists() {
        eprintln!("{:?} does not exist!", config_fp);
        exit(1);
    }
    let data =
        fs::read_to_string(config_fp.clone()).expect(&format!("Unable to read {:?}", config_fp));
    let t: Task = serde_yaml::from_str(&data).expect("Unable to open config file");
    print!("{:#?}", t);
    let sleep_duration = Duration::from_millis(t.ping_sleep_millis);
    wakeup(
        &t.host,
        &t.ping_cmd,
        &sleep_duration,
        t.wakeup_instructions,
        t.ssh.clone(),
    )?;
    let no_args: Vec<String> = vec![];

    for i in t.instructions {
        match i.execution_side {
            wakenrun::ExecutionSide::Local => run_cmd(i.command, &no_args, None, true, true)?,
            wakenrun::ExecutionSide::Remote => run_remote_cmd(&t.host, t.ssh.clone(), i.command)?,
        };
    }
    shutdown(
        &t.host,
        &t.ping_cmd,
        &sleep_duration,
        t.shutdown_instructions,
        t.ssh,
    )?;
    Ok(())
}
