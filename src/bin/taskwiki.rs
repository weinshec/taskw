use clap::Parser;
use env_logger::Env;

use taskw::cli::{task_from_stdin, Cli, Commands};
use taskw::config::Config;
use taskw::hooks::Hooks;

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
