use clap::{Parser, Subcommand};
use env_logger::Env;
use log::debug;
use std::str::FromStr;
use taskw::Task;

/// taskwarrior hooks into vimwiki
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    /// Enable debug logging to stderr
    #[clap(short, long)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// called with taskwarriors on-add hook
    Add,
    /// called with taskwarriors on-modify hook
    Modify,
}

fn main() -> Result<(), &'static str> {
    let cli = Cli::parse();
    let env = match cli.debug {
        true => Env::default().filter_or("RUST_LOG", "DEBUG"),
        false => Env::default().filter_or("RUST_LOG", "ERROR"),
    };
    env_logger::init_from_env(env);

    match &cli.command {
        Commands::Add => on_add(),
        Commands::Modify => on_modify(),
    }
}

fn task_from_stdin() -> Result<Task, &'static str> {
    let mut json = String::new();
    std::io::stdin()
        .read_line(&mut json)
        .map_err(|_| "cannot read from stdin")?;
    Task::from_str(json.trim())
}

fn on_add() -> Result<(), &'static str> {
    let new_task = task_from_stdin()?;
    debug!("task added = {:#?}", new_task);
    println!("{}", new_task);
    Ok(())
}

fn on_modify() -> Result<(), &'static str> {
    let original_task = task_from_stdin()?;
    debug!("task original = {:#?}", original_task);
    let modified_task = task_from_stdin()?;
    debug!("task modified = {:#?}", modified_task);
    println!("{}", modified_task);
    Ok(())
}
