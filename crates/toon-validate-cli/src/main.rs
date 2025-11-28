mod analyze;
mod check;
mod commands;
mod profile;

use anyhow::Result;
use clap::Parser;
use commands::{Cli, Commands};
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Analyze { path, format, json } => {
            let input_format = format.map(|f| f.to_input_format());
            analyze::analyze_file(&path, input_format, json)?;
        }
        Commands::Profile {
            dir,
            extensions,
            format,
            json,
        } => {
            let input_format = format.map(|f| f.to_input_format());
            profile::profile_directory(&dir, extensions, input_format, json)?;
        }
        Commands::Check { path, format, json } => {
            let input_format = format.map(|f| f.to_input_format());
            check::check_file(&path, input_format, json)?;
        }
    }
    
    Ok(())
}