use crate::{error::{Result, TqError}, value::{Table, Value}};
use std::collections::HashMap;

pub struct ToonParser {
    lines: Vec<String>,
    current: usize,
}

impl ToonParser {
    pub fn parse(input: &str) -> Result<Value> {
        let mut parser = ToonParser {
            lines: input.lines().map(String::from).collect(),
            current: 0,
        };
        parser.parse_value(0)
    }
    
    fn parse_value(&mut self, indent: usize) -> Result<Value> {
        let mut obj = HashMap::new();
        
        while self.current < self.lines.len() {
            let line = &self.lines[self.current];
            let line_indent = Self::count_indent(line);
            
            if line_indent < indent {
                break;
            }
            
            if line_indent > indent {
                continue;
            }
            
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                self.current += 1;
                continue;
            }
            
            if let Some(table_match) = Self::parse_table_header(trimmed) {
                self.current += 1;
                let table = self.parse_table(table_match.0, table_match.1, indent)?;
                obj.insert(table.name.clone(), Value::Table(table));
            } else if let Some((key, value)) = Self::parse_key_value(trimmed) {
                self.current += 1;
                // Check if this is a nested object
                if matches!(value, Value::Null) && self.current < self.lines.len() {
                    let next_line_indent = Self::count_indent(&self.lines[self.current]);
                    if next_line_indent > indent {
                        let nested_value = self.parse_value(indent + 2)?;
                        obj.insert(key, nested_value);
                    } else {
                        obj.insert(key, value);
                    }
                } else {
                    obj.insert(key, value);
                }
            } else {
                return Err(TqError::Parse {
                    line: self.current + 1,
                    message: format!("Invalid syntax: {}", trimmed),
                });
            }
        }
        
        Ok(Value::Object(obj))
    }
    
    fn parse_table(&mut self, name: String, declared_rows: usize, parent_indent: usize) -> Result<Table> {
        let mut rows = Vec::new();
        let expected_indent = parent_indent + 2;
        
        while self.current < self.lines.len() && rows.len() < declared_rows {
            let line = &self.lines[self.current];
            let line_indent = Self::count_indent(line);
            
            if line_indent < expected_indent {
                break;
            }
            
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                self.current += 1;
                continue;
            }
            
            if trimmed.starts_with("- ") {
                // Check if there's a field on the same line as the dash
                let after_dash = &trimmed[2..];
                if !after_dash.is_empty() {
                    // Parse the field on the same line
                    let mut row = HashMap::new();
                    if let Some((key, value)) = Self::parse_key_value(after_dash) {
                        row.insert(key, value);
                    }
                    self.current += 1;
                    // Then parse any additional fields on following lines
                    let additional_fields = self.parse_table_row(expected_indent + 2)?;
                    row.extend(additional_fields);
                    rows.push(row);
                } else {
                    // Dash is on its own line, fields are on following lines
                    self.current += 1;
                    let row = self.parse_table_row(expected_indent + 2)?;
                    rows.push(row);
                }
            } else {
                self.current += 1;
            }
        }
        
        if rows.len() != declared_rows {
            return Err(TqError::TableRowMismatch {
                name: name.clone(),
                declared: declared_rows,
                actual: rows.len(),
            });
        }
        
        Ok(Table {
            name,
            declared_rows,
            rows,
        })
    }
    
    fn parse_table_row(&mut self, indent: usize) -> Result<HashMap<String, Value>> {
        let mut row = HashMap::new();
        
        while self.current < self.lines.len() {
            let line = &self.lines[self.current];
            let line_indent = Self::count_indent(line);
            
            // Stop if we've gone back to a less indented line
            if line_indent < indent {
                break;
            }
            
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                self.current += 1;
                continue;
            }
            
            // Stop if we hit another row marker
            if trimmed.starts_with("- ") {
                break;
            }
            
            // Only parse fields at the expected indent level
            if line_indent == indent {
                if let Some((key, value)) = Self::parse_key_value(trimmed) {
                    self.current += 1;
                    row.insert(key, value);
                } else {
                    break;
                }
            } else {
                // Skip lines that are more indented (could be nested structures)
                self.current += 1;
            }
        }
        
        Ok(row)
    }
    
    fn parse_table_header(line: &str) -> Option<(String, usize)> {
        if let Some(bracket_pos) = line.find('[') {
            if let Some(colon_pos) = line.rfind(':') {
                if bracket_pos < colon_pos {
                    let name = line[..bracket_pos].trim().to_string();
                    let num_str = line[bracket_pos + 1..colon_pos]
                        .trim_end_matches(']')
                        .trim();
                    if let Ok(num) = num_str.parse::<usize>() {
                        return Some((name, num));
                    }
                }
            }
        }
        None
    }
    
    fn parse_key_value(line: &str) -> Option<(String, Value)> {
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim().to_string();
            let value_str = line[colon_pos + 1..].trim();
            let value = if value_str.is_empty() {
                Value::Null  // This indicates a nested object
            } else {
                Self::parse_simple_value(value_str)
            };
            return Some((key, value));
        }
        None
    }
    
    fn parse_simple_value(s: &str) -> Value {
        let trimmed = s.trim();
        
        if trimmed == "null" {
            return Value::Null;
        }
        
        if trimmed == "true" {
            return Value::Bool(true);
        }
        
        if trimmed == "false" {
            return Value::Bool(false);
        }
        
        if (trimmed.starts_with('"') && trimmed.ends_with('"')) ||
           (trimmed.starts_with('\'') && trimmed.ends_with('\'')) {
            let unquoted = &trimmed[1..trimmed.len() - 1];
            return Value::String(unquoted.to_string());
        }
        
        if let Ok(num) = trimmed.parse::<f64>() {
            return Value::Number(num);
        }
        
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let array_content = &trimmed[1..trimmed.len() - 1];
            let items: Vec<Value> = array_content
                .split(',')
                .map(|item| Self::parse_simple_value(item.trim()))
                .collect();
            return Value::Array(items);
        }
        
        Value::String(trimmed.to_string())
    }
    
    fn count_indent(line: &str) -> usize {
        line.chars().take_while(|c| *c == ' ').count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_values() {
        let input = r#"name: "test"
age: 42
active: true
empty: null"#;
        let result = ToonParser::parse(input).unwrap();
        
        if let Value::Object(obj) = result {
            assert_eq!(obj.get("name"), Some(&Value::String("test".to_string())));
            assert_eq!(obj.get("age"), Some(&Value::Number(42.0)));
            assert_eq!(obj.get("active"), Some(&Value::Bool(true)));
            assert_eq!(obj.get("empty"), Some(&Value::Null));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_nested_object() {
        let input = r#"user:
  name: "Alice"
  age: 30
  settings:
    theme: "dark"
    notifications: true"#;
        let result = ToonParser::parse(input).unwrap();
        
        if let Value::Object(obj) = result {
            if let Some(Value::Object(user)) = obj.get("user") {
                assert_eq!(user.get("name"), Some(&Value::String("Alice".to_string())));
                assert_eq!(user.get("age"), Some(&Value::Number(30.0)));
                
                if let Some(Value::Object(settings)) = user.get("settings") {
                    assert_eq!(settings.get("theme"), Some(&Value::String("dark".to_string())));
                    assert_eq!(settings.get("notifications"), Some(&Value::Bool(true)));
                } else {
                    panic!("Expected settings object");
                }
            } else {
                panic!("Expected user object");
            }
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_table() {
        let input = r#"users[2]:
  - id: 1
    name: "Alice"
  - id: 2
    name: "Bob""#;
        let result = ToonParser::parse(input).unwrap();
        
        if let Value::Object(obj) = result {
            if let Some(Value::Table(table)) = obj.get("users") {
                assert_eq!(table.name, "users");
                assert_eq!(table.declared_rows, 2);
                assert_eq!(table.rows.len(), 2);
                
                assert_eq!(table.rows[0].get("id"), Some(&Value::Number(1.0)));
                assert_eq!(table.rows[0].get("name"), Some(&Value::String("Alice".to_string())));
                assert_eq!(table.rows[1].get("id"), Some(&Value::Number(2.0)));
                assert_eq!(table.rows[1].get("name"), Some(&Value::String("Bob".to_string())));
            } else {
                panic!("Expected table, got: {:?}", obj);
            }
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_array() {
        let input = r#"tags: ["rust", "cli", "tool"]
numbers: [1, 2, 3]"#;
        let result = ToonParser::parse(input).unwrap();
        
        if let Value::Object(obj) = result {
            if let Some(Value::Array(tags)) = obj.get("tags") {
                assert_eq!(tags.len(), 3);
                assert_eq!(tags[0], Value::String("rust".to_string()));
                assert_eq!(tags[1], Value::String("cli".to_string()));
                assert_eq!(tags[2], Value::String("tool".to_string()));
            } else {
                panic!("Expected tags array");
            }
            
            if let Some(Value::Array(numbers)) = obj.get("numbers") {
                assert_eq!(numbers.len(), 3);
                assert_eq!(numbers[0], Value::Number(1.0));
                assert_eq!(numbers[1], Value::Number(2.0));
                assert_eq!(numbers[2], Value::Number(3.0));
            } else {
                panic!("Expected numbers array");
            }
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_with_comments() {
        let input = r#"# This is a comment
name: "test"
# Another comment
active: true"#;
        let result = ToonParser::parse(input).unwrap();
        
        if let Value::Object(obj) = result {
            assert_eq!(obj.len(), 2);
            assert_eq!(obj.get("name"), Some(&Value::String("test".to_string())));
            assert_eq!(obj.get("active"), Some(&Value::Bool(true)));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_table_row_count_validation() {
        let input = r#"users[3]:
  - id: 1
    name: "Alice"
  - id: 2
    name: "Bob""#;
        let result = ToonParser::parse(input);
        
        assert!(result.is_err());
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(error_msg.contains("declared with 3 rows but found 2"));
        }
    }
}