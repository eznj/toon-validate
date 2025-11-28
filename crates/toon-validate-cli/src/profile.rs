use anyhow::{Context, Result};
use prettytable::{row, Table};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use toon_validate_core::{InputFormat, Parser, TokenEstimator};
use walkdir::WalkDir;

#[derive(Serialize, Deserialize)]
pub struct ProfileResult {
    pub directory: String,
    pub total_files: usize,
    pub total_tokens: usize,
    pub files: Vec<FileProfile>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileProfile {
    pub path: String,
    pub tokens: usize,
    pub format: String,
}

pub fn profile_directory(
    dir: &Path,
    extensions: Vec<String>,
    format: Option<InputFormat>,
    json_output: bool,
) -> Result<()> {
    let mut files = Vec::new();
    let mut total_tokens = 0;
    
    let extensions: Vec<String> = if extensions.is_empty() {
        vec!["toon".to_string(), "json".to_string()]
    } else {
        extensions
    };
    
    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        
        let path = entry.path();
        let should_process = if let Some(ext) = path.extension() {
            extensions.iter().any(|e| e == &ext.to_string_lossy())
        } else {
            false
        };
        
        if !should_process {
            continue;
        }
        
        match process_file(path, format) {
            Ok(profile) => {
                total_tokens += profile.tokens;
                files.push(profile);
            }
            Err(e) => {
                eprintln!("Warning: Failed to process {}: {}", path.display(), e);
            }
        }
    }
    
    // Sort by token count descending
    files.sort_by(|a, b| b.tokens.cmp(&a.tokens));
    
    if json_output {
        let result = ProfileResult {
            directory: dir.display().to_string(),
            total_files: files.len(),
            total_tokens,
            files,
        };
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("\nDirectory Profile: {}", dir.display());
        println!("Total Files: {}", files.len());
        println!("Total Estimated Tokens: {}", total_tokens);
        println!();
        
        if !files.is_empty() {
            let mut table = Table::new();
            table.add_row(row!["File", "Tokens", "Format", "% of Total"]);
            
            for file in files.iter().take(20) {
                let percentage = if total_tokens > 0 {
                    format!("{:.1}%", (file.tokens as f64 / total_tokens as f64) * 100.0)
                } else {
                    "0.0%".to_string()
                };
                
                table.add_row(row![
                    file.path,
                    file.tokens,
                    file.format,
                    percentage
                ]);
            }
            
            if files.len() > 20 {
                table.add_row(row![
                    format!("... and {} more files", files.len() - 20),
                    "",
                    "",
                    ""
                ]);
            }
            
            table.printstd();
        } else {
            println!("No matching files found.");
        }
    }
    
    Ok(())
}

fn process_file(path: &Path, format: Option<InputFormat>) -> Result<FileProfile> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    
    let input_format = format.unwrap_or_else(|| Parser::detect_format(&content));
    
    let value = Parser::parse(&content, input_format)
        .with_context(|| format!("Failed to parse file: {}", path.display()))?;
    
    let tokens = TokenEstimator::estimate(&value);
    
    Ok(FileProfile {
        path: path.display().to_string(),
        tokens,
        format: match input_format {
            InputFormat::Toon => "toon".to_string(),
            InputFormat::Json => "json".to_string(),
        },
    })
}