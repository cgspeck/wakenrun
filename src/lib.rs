use std::{fs, path::PathBuf, process::exit};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

fn default_ssh_command() -> String {
    String::from("/usr/bin/ssh")
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SshInstructions {
    #[serde(default = "default_ssh_command")]
    pub ssh_cmd: String,
    pub ssh_identity_file: Option<String>,
    pub ssh_port: Option<u16>,
    pub ssh_user: Option<String>,
}

impl Default for SshInstructions {
    fn default() -> Self {
        Self {
            ssh_cmd: default_ssh_command(),
            ssh_identity_file: Default::default(),
            ssh_port: Default::default(),
            ssh_user: Default::default(),
        }
    }
}

fn default_wakeup_boot_timeout_secs() -> u64 {
    120
}

fn default_wakeup_enabled() -> bool {
    true
}

fn default_wakeup_validate_ping() -> bool {
    true
}

fn default_wakeup_validate_ssh_connection() -> bool {
    true
}

fn default_ping_cmd() -> String {
    String::from("/usr/bin/ping")
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct WakeupInstructions {
    #[serde(default = "default_wakeup_enabled")]
    pub enabled: bool,
    pub mac: String,
    #[serde(default = "default_wakeup_boot_timeout_secs")]
    pub boot_timeout_secs: u64,
    #[serde(default = "default_wakeup_validate_ping")]
    pub validate_ping: bool,
    #[serde(default = "default_wakeup_validate_ssh_connection")]
    pub validate_ssh_connection: bool,
}

impl Default for WakeupInstructions {
    fn default() -> Self {
        Self {
            enabled: default_wakeup_enabled(),
            mac: "INSERT-REMOTE-MAC-ADDRESS".into(),
            boot_timeout_secs: default_wakeup_boot_timeout_secs(),
            validate_ping: default_wakeup_validate_ping(),
            validate_ssh_connection: default_wakeup_validate_ssh_connection(),
        }
    }
}

fn default_after_shutdown_remote() -> bool {
    true
}

fn default_after_shutdown_cmd() -> String {
    String::from("sudo /usr/bin/shutdown now")
}

fn default_after_validate_shutdown() -> bool {
    true
}

fn default_after_shutdown_timeout_secs() -> u64 {
    120
}

fn default_ping_sleep_millis() -> u64 {
    500
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ShutdownInstructions {
    #[serde(default = "default_after_shutdown_remote")]
    pub shutdown_remote: bool,
    #[serde(default = "default_after_shutdown_cmd")]
    pub shutdown_cmd: String,
    #[serde(default = "default_after_validate_shutdown")]
    pub validate_shutdown: bool,
    #[serde(default = "default_after_shutdown_timeout_secs")]
    pub shutdown_timeout_secs: u64,
}

impl Default for ShutdownInstructions {
    fn default() -> Self {
        Self {
            shutdown_remote: default_after_shutdown_remote(),
            shutdown_cmd: default_after_shutdown_cmd(),
            validate_shutdown: default_after_validate_shutdown(),
            shutdown_timeout_secs: default_after_shutdown_timeout_secs(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, ValueEnum)]
pub enum ExecutionSide {
    Local,
    Remote,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProcessInstruction {
    pub execution_side: ExecutionSide,
    pub command: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Task {
    pub host: String,
    pub ssh: SshInstructions,
    pub wakeup_instructions: WakeupInstructions,
    pub instructions: Vec<ProcessInstruction>,
    pub shutdown_instructions: ShutdownInstructions,
    #[serde(default = "default_ping_cmd")]
    pub ping_cmd: String,
    #[serde(default = "default_ping_sleep_millis")]
    pub ping_sleep_millis: u64,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            host: "remote-hostname".into(),
            ssh: Default::default(),
            wakeup_instructions: Default::default(),
            instructions: Default::default(),
            shutdown_instructions: Default::default(),
            ping_cmd: default_ping_cmd(),
            ping_sleep_millis: default_ping_sleep_millis(),
        }
    }
}

pub fn generate_sample_config(config_fp: &PathBuf) -> anyhow::Result<()> {
    match config_fp.exists() {
        true => {
            eprintln!("Not overwriting existing config!");
            exit(1);
        }
        false => {
            let mut sample_config = Task::default();
            sample_config.instructions = vec![
                ProcessInstruction {
                    execution_side: ExecutionSide::Local,
                    command: "echo 'hello world from local side'".into(),
                },
                ProcessInstruction {
                    execution_side: ExecutionSide::Remote,
                    command: "echo 'hello world from remote side'".into(),
                },
            ];

            let fh = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(config_fp)?;
            serde_yaml::to_writer(fh, &sample_config)?;
            Ok(())
        }
    }
}
