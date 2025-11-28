use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "tq")]
#[command(about = "TOON Analyzer - Analyze and validate TOON and JSON files", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Analyze a single file and show token breakdown
    Analyze {
        /// Path to the file to analyze
        path: PathBuf,
        
        /// Input format (toon or json)
        #[arg(long = "in", value_enum)]
        format: Option<Format>,
        
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    
    /// Profile all files in a directory
    Profile {
        /// Directory to profile
        dir: PathBuf,
        
        /// File extensions to include (can be specified multiple times)
        #[arg(long = "ext")]
        extensions: Vec<String>,
        
        /// Input format (toon or json)
        #[arg(long = "in", value_enum)]
        format: Option<Format>,
        
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    
    /// Validate TOON structure and table consistency
    Check {
        /// Path to the file to check
        path: PathBuf,
        
        /// Input format (toon or json)
        #[arg(long = "in", value_enum)]
        format: Option<Format>,
        
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Format {
    Toon,
    Json,
}

impl Format {
    pub fn to_input_format(self) -> toon_validate_core::InputFormat {
        match self {
            Format::Toon => toon_validate_core::InputFormat::Toon,
            Format::Json => toon_validate_core::InputFormat::Json,
        }
    }
}