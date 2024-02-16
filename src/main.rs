use std::{fs, path::PathBuf};

use clap::Parser;
use wakenrun::Task;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(value_name = "FILE")]
    config: PathBuf,
}

fn main() {
    let config_fp = Cli::parse().config;

    if !config_fp.exists() {
        panic!("{:?} does not exist!", config_fp)
    }
    let data =
        fs::read_to_string(config_fp.clone()).expect(&format!("Unable to read {:?}", config_fp));
    let task: Task = serde_yaml::from_str(&data).expect("Unable to open config file");
    println!("{:#?}", task);
}
