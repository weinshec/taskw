use clap::{Parser, Subcommand};
use env_logger::Env;
use std::str::FromStr;
use taskw::config::Config;
use taskw::hooks::Hooks;
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
    let cfg = Config::default().to_static();
    let env = match cli.debug {
        true => Env::default().filter_or("RUST_LOG", "DEBUG"),
        false => Env::default().filter_or("RUST_LOG", "ERROR"),
    };
    env_logger::init_from_env(env);

    let hooks = Hooks::with_config(cfg);
    match &cli.command {
        Commands::Add => {
            let added_task = task_from_stdin()?;
            let (task, feedback) = hooks.on_add(added_task)?;
            println!("{}\n{}", task, feedback);
        }
        Commands::Modify => {
            let original_task = task_from_stdin()?;
            let modified_task = task_from_stdin()?;
            let (task, feedback) = hooks.on_modify(original_task, modified_task)?;
            println!("{}\n{}", task, feedback);
        }
    }

    Ok(())
}

fn task_from_stdin() -> Result<Task, &'static str> {
    let mut json = String::new();
    std::io::stdin()
        .read_line(&mut json)
        .map_err(|_| "cannot read from stdin")?;
    Task::from_str(json.trim())
}
