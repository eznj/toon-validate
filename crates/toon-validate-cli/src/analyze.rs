use anyhow::{Context, Result};
use prettytable::{row, Table};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use toon_validate_core::{InputFormat, Parser, TokenBreakdown, TokenEstimator};

#[derive(Serialize, Deserialize)]
pub struct AnalysisResult {
    pub file: String,
    pub format: String,
    pub total_tokens: usize,
    pub breakdown: TokenBreakdownJson,
}

#[derive(Serialize, Deserialize)]
pub struct TokenBreakdownJson {
    pub keys: usize,
    pub strings: usize,
    pub primitives: usize,
    pub structure: usize,
    pub tables: usize,
    pub table_rows: usize,
}

impl From<&TokenBreakdown> for TokenBreakdownJson {
    fn from(breakdown: &TokenBreakdown) -> Self {
        TokenBreakdownJson {
            keys: breakdown.keys,
            strings: breakdown.strings,
            primitives: breakdown.primitives,
            structure: breakdown.structure,
            tables: breakdown.tables,
            table_rows: breakdown.table_rows,
        }
    }
}

pub fn analyze_file(
    path: &Path,
    format: Option<InputFormat>,
    json_output: bool,
) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    
    let input_format = format.unwrap_or_else(|| Parser::detect_format(&content));
    
    let value = Parser::parse(&content, input_format)
        .with_context(|| format!("Failed to parse file: {}", path.display()))?;
    
    let breakdown = TokenEstimator::estimate_breakdown(&value);
    let total_tokens = breakdown.total();
    
    if json_output {
        let result = AnalysisResult {
            file: path.display().to_string(),
            format: match input_format {
                InputFormat::Toon => "toon".to_string(),
                InputFormat::Json => "json".to_string(),
            },
            total_tokens,
            breakdown: TokenBreakdownJson::from(&breakdown),
        };
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("\nFile Analysis: {}", path.display());
        println!("Format: {:?}", input_format);
        println!("Total Estimated Tokens: {}", total_tokens);
        
        let mut table = Table::new();
        table.add_row(row!["Component", "Tokens", "Percentage"]);
        
        if total_tokens > 0 {
            table.add_row(row![
                "Keys",
                breakdown.keys,
                format!("{:.1}%", (breakdown.keys as f64 / total_tokens as f64) * 100.0)
            ]);
            table.add_row(row![
                "Strings",
                breakdown.strings,
                format!("{:.1}%", (breakdown.strings as f64 / total_tokens as f64) * 100.0)
            ]);
            table.add_row(row![
                "Primitives",
                breakdown.primitives,
                format!("{:.1}%", (breakdown.primitives as f64 / total_tokens as f64) * 100.0)
            ]);
            table.add_row(row![
                "Structure",
                breakdown.structure,
                format!("{:.1}%", (breakdown.structure as f64 / total_tokens as f64) * 100.0)
            ]);
            if breakdown.tables > 0 {
                table.add_row(row![
                    "Tables",
                    format!("{} ({}rows)", breakdown.tables, breakdown.table_rows),
                    format!("{:.1}%", (breakdown.tables as f64 / total_tokens as f64) * 100.0)
                ]);
            }
        }
        
        table.printstd();
    }
    
    Ok(())
}