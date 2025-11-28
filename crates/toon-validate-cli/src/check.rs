use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use toon_validate_core::{InputFormat, Parser, TqError, Validator};

#[derive(Serialize, Deserialize)]
pub struct CheckResult {
    pub file: String,
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn check_file(
    path: &Path,
    format: Option<InputFormat>,
    json_output: bool,
) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    
    let input_format = format.unwrap_or_else(|| Parser::detect_format(&content));
    
    let parse_result = Parser::parse(&content, input_format);
    
    let (value, parse_error) = match parse_result {
        Ok(v) => (Some(v), None),
        Err(e) => (None, Some(e)),
    };
    
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    
    if let Some(e) = parse_error {
        errors.push(format!("Parse error: {}", e));
        
        if json_output {
            let result = CheckResult {
                file: path.display().to_string(),
                valid: false,
                errors,
                warnings,
            };
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("\nValidation Result: {}", path.display());
            println!("Status: INVALID");
            println!("Errors:");
            for error in &errors {
                println!("  - {}", error);
            }
        }
        
        // Parse error is exit code 1, validation error is exit code 2
        let exit_code = match &e {
            TqError::TableRowMismatch { .. } | 
            TqError::TableSchemaInconsistent { .. } | 
            TqError::Validation(_) => 2,
            _ => 1,
        };
        std::process::exit(exit_code);
    }
    
    let value = value.unwrap();
    
    // Perform validation
    match Validator::validate(&value) {
        Ok(_) => {}
        Err(e) => {
            match &e {
                TqError::TableRowMismatch { .. } | 
                TqError::TableSchemaInconsistent { .. } => {
                    errors.push(format!("Validation error: {}", e));
                }
                _ => {
                    errors.push(format!("Error: {}", e));
                }
            }
        }
    }
    
    // Check structure for warnings
    let structural_issues = Validator::check_structure(&value);
    warnings.extend(structural_issues);
    
    let is_valid = errors.is_empty();
    
    if json_output {
        let result = CheckResult {
            file: path.display().to_string(),
            valid: is_valid,
            errors,
            warnings,
        };
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("\nValidation Result: {}", path.display());
        println!("Format: {:?}", input_format);
        println!("Status: {}", if is_valid { "VALID" } else { "INVALID" });
        
        if !errors.is_empty() {
            println!("\nErrors:");
            for error in &errors {
                println!("  - {}", error);
            }
        }
        
        if !warnings.is_empty() {
            println!("\nWarnings:");
            for warning in &warnings {
                println!("  - {}", warning);
            }
        }
        
        if is_valid && errors.is_empty() && warnings.is_empty() {
            println!("\nNo issues found.");
        }
    }
    
    if !is_valid {
        std::process::exit(2);
    }
    
    Ok(())
}