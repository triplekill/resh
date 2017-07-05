#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::collections::BTreeMap;

use std::env;
use std::fs::File;
use std::io::prelude::*;

macro_rules! die(
    ($($arg:tt)*) => { {
        writeln!(std::io::stderr(), $($arg)*)
            .expect("Failed to print to stderr");
        std::process::exit(1);
    } }
);

#[derive(Deserialize)]
struct Config {
    commands: BTreeMap<String,String>,
}

fn read_config<P: AsRef<std::path::Path>>(path: P) -> Result<Config, Box<std::error::Error>> {
    let mut contents = String::new();

    File::open(path)?
        .read_to_string(&mut contents)?;

    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

fn run_command(command: &str) -> Result<i32, Box<std::error::Error>> {
    let mut child = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(command)
        .spawn()?;

    child
        .wait()?
        .code()
        .ok_or(Box::new(std::io::Error::last_os_error()))
}

fn main() {
    let program_name = std::env::args().nth(0).unwrap_or("resh".to_string());

    let command_alias = match std::env::args().nth(1) {
        Some(alias) => alias,
        None => { die!("Usage: {} <command alias>", program_name) }
    };

    let config_file = env::var("RESH_CONFIG")
        .unwrap_or_else(|_| {"/etc/resh.toml".to_string()});

    let config: Config = read_config(&config_file).
        unwrap_or_else(|e| { die!("Failed to read {}: {}", config_file, e); });

    let full_command = match config.commands.get(&command_alias) {
        Some(cmd) => cmd,
        None => { die!("Undefined command alias: {}", command_alias) },
    };

    let exitcode = run_command(&full_command)
        .unwrap_or(1);

    std::process::exit(exitcode);
}
