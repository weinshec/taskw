use clap::{Parser, Subcommand};
use std::str::FromStr;

use crate::Task;

/// taskwarrior hooks into vimwiki
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    /// Enable debug logging to stderr
    #[clap(short, long)]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// called with taskwarriors on-add hook
    Add,
    /// called with taskwarriors on-modify hook
    Modify,
}

pub fn task_from_stdin() -> Result<Task, &'static str> {
    let mut json = String::new();
    std::io::stdin()
        .read_line(&mut json)
        .map_err(|_| "cannot read from stdin")?;
    Task::from_str(json.trim())
}
