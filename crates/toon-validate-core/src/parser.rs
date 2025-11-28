use crate::{
    error::{Result, TqError},
    json_parser::JsonParser,
    toon_parser::ToonParser,
    value::Value,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputFormat {
    Toon,
    Json,
}

impl InputFormat {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "toon" => Ok(InputFormat::Toon),
            "json" => Ok(InputFormat::Json),
            _ => Err(TqError::InvalidFormat(format!("Unknown format: {}", s))),
        }
    }
}

pub struct Parser;

impl Parser {
    pub fn parse(input: &str, format: InputFormat) -> Result<Value> {
        match format {
            InputFormat::Toon => ToonParser::parse(input),
            InputFormat::Json => JsonParser::parse(input),
        }
    }
    
    pub fn detect_format(input: &str) -> InputFormat {
        let trimmed = input.trim();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            InputFormat::Json
        } else {
            InputFormat::Toon
        }
    }
}