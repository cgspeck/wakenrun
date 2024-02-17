use clap::ValueEnum;
use serde::Deserialize;

fn default_ssh_command() -> String {
    String::from("/usr/bin/ssh")
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct SshInstructions {
    #[serde(default = "default_ssh_command")]
    pub ssh_cmd: String,
    pub ssh_identity_file: Option<String>,
    pub ssh_port: Option<u16>,
    pub ssh_user: Option<String>,
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

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
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

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
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

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, ValueEnum)]
pub enum ExecutionSide {
    Local,
    Remote,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct ProcessInstruction {
    pub execution_side: ExecutionSide,
    pub command: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Task {
    pub host: String,
    pub ssh: SshInstructions,
    pub wakeup_instructions: WakeupInstructions,
    pub instructions: Vec<ProcessInstruction>,
    pub shutdown_instructions: ShutdownInstructions,
    #[serde(default = "default_ping_cmd")]
    pub ping_cmd: String,
}
