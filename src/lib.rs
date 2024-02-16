pub struct WakeupInstructions {
    pub enabled: bool,
    pub mac: String,
    pub boot_timeout_secs: u16,
    pub validate_ping: bool,
    pub validate_ssh_port: bool,
}

pub struct AfterInstructions {
    pub shutdown_remote: bool,
    pub shutdown_cmd: String,
    pub validate_shutdown: bool,
    pub shutdown_timeout_secs: u16,
}

pub enum ExecutionSide {
    Local,
    Remote,
}

pub struct ProcessInstruction {
    pub execution_side: ExecutionSide,
    pub command: String,
}

pub struct Task {
    pub host: String,
    pub ssh_port: u16,
    pub ssh_cmd: String,
    pub wakeup_instructions: WakeupInstructions,
    pub instructions: Vec<ProcessInstruction>,
    pub after_instructions: AfterInstructions
}