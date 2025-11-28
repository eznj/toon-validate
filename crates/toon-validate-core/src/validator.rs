use crate::{
    error::{Result, TqError},
    value::{Table, Value},
};
use std::collections::{HashMap, HashSet};

pub struct Validator;

impl Validator {
    pub fn validate(value: &Value) -> Result<()> {
        match value {
            Value::Object(obj) => Self::validate_object(obj),
            Value::Table(table) => Self::validate_table(table),
            Value::Array(arr) => {
                for item in arr {
                    Self::validate(item)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    
    fn validate_object(obj: &HashMap<String, Value>) -> Result<()> {
        for value in obj.values() {
            Self::validate(value)?;
        }
        Ok(())
    }
    
    fn validate_table(table: &Table) -> Result<()> {
        // Check row count matches declaration
        if table.rows.len() != table.declared_rows {
            return Err(TqError::TableRowMismatch {
                name: table.name.clone(),
                declared: table.declared_rows,
                actual: table.rows.len(),
            });
        }
        
        // Check schema consistency across rows
        if !table.rows.is_empty() {
            let mut schemas: Vec<HashSet<String>> = Vec::new();
            
            for row in &table.rows {
                let schema: HashSet<String> = row.keys().cloned().collect();
                schemas.push(schema);
            }
            
            // Check if all rows have the same set of keys
            let first_schema = &schemas[0];
            for (idx, schema) in schemas.iter().enumerate().skip(1) {
                if schema != first_schema {
                    let missing: Vec<_> = first_schema.difference(schema).collect();
                    let extra: Vec<_> = schema.difference(first_schema).collect();
                    
                    let mut message = format!("Row {} has different schema. ", idx + 1);
                    if !missing.is_empty() {
                        message.push_str(&format!("Missing fields: {:?}. ", missing));
                    }
                    if !extra.is_empty() {
                        message.push_str(&format!("Extra fields: {:?}. ", extra));
                    }
                    
                    return Err(TqError::TableSchemaInconsistent {
                        name: table.name.clone(),
                        message,
                    });
                }
            }
        }
        
        // Recursively validate values in rows
        for row in &table.rows {
            for value in row.values() {
                Self::validate(value)?;
            }
        }
        
        Ok(())
    }
    
    pub fn check_structure(value: &Value) -> Vec<String> {
        let mut issues = Vec::new();
        Self::check_structure_recursive(value, "", &mut issues);
        issues
    }
    
    fn check_structure_recursive(value: &Value, path: &str, issues: &mut Vec<String>) {
        match value {
            Value::Object(obj) => {
                if obj.is_empty() {
                    let prefix = if path.is_empty() { 
                        String::new() 
                    } else { 
                        format!("{}: ", path) 
                    };
                    issues.push(format!("{}Empty object", prefix));
                }
                for (key, val) in obj {
                    let new_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    Self::check_structure_recursive(val, &new_path, issues);
                }
            }
            Value::Array(arr) => {
                if arr.is_empty() {
                    issues.push(format!("{}: Empty array", path));
                }
                for (idx, val) in arr.iter().enumerate() {
                    let new_path = format!("{}[{}]", path, idx);
                    Self::check_structure_recursive(val, &new_path, issues);
                }
            }
            Value::Table(table) => {
                if table.rows.is_empty() && table.declared_rows > 0 {
                    issues.push(format!("{}: Table declared with {} rows but is empty", path, table.declared_rows));
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Table;

    #[test]
    fn test_validate_primitives() {
        assert!(Validator::validate(&Value::Null).is_ok());
        assert!(Validator::validate(&Value::Bool(true)).is_ok());
        assert!(Validator::validate(&Value::Number(42.0)).is_ok());
        assert!(Validator::validate(&Value::String("test".to_string())).is_ok());
    }

    #[test]
    fn test_validate_array() {
        let arr = Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::String("test".to_string()),
        ]);
        assert!(Validator::validate(&arr).is_ok());
    }

    #[test]
    fn test_validate_object() {
        let mut obj = HashMap::new();
        obj.insert("name".to_string(), Value::String("Alice".to_string()));
        obj.insert("age".to_string(), Value::Number(30.0));
        obj.insert("active".to_string(), Value::Bool(true));
        let value = Value::Object(obj);
        
        assert!(Validator::validate(&value).is_ok());
    }

    #[test]
    fn test_validate_table_correct_rows() {
        let mut row1 = HashMap::new();
        row1.insert("id".to_string(), Value::Number(1.0));
        row1.insert("name".to_string(), Value::String("Alice".to_string()));
        
        let mut row2 = HashMap::new();
        row2.insert("id".to_string(), Value::Number(2.0));
        row2.insert("name".to_string(), Value::String("Bob".to_string()));
        
        let table = Table {
            name: "users".to_string(),
            declared_rows: 2,
            rows: vec![row1, row2],
        };
        
        assert!(Validator::validate(&Value::Table(table)).is_ok());
    }

    #[test]
    fn test_validate_table_row_mismatch() {
        let mut row1 = HashMap::new();
        row1.insert("id".to_string(), Value::Number(1.0));
        
        let table = Table {
            name: "users".to_string(),
            declared_rows: 3, // Declared 3 but only 1 row
            rows: vec![row1],
        };
        
        let result = Validator::validate(&Value::Table(table));
        assert!(result.is_err());
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(error_msg.contains("declared"));
            assert!(error_msg.contains("3"));
            assert!(error_msg.contains("1"));
        }
    }

    #[test]
    fn test_validate_table_schema_inconsistent() {
        let mut row1 = HashMap::new();
        row1.insert("id".to_string(), Value::Number(1.0));
        row1.insert("name".to_string(), Value::String("Alice".to_string()));
        
        let mut row2 = HashMap::new();
        row2.insert("id".to_string(), Value::Number(2.0));
        row2.insert("email".to_string(), Value::String("bob@example.com".to_string()));
        // Missing "name" field, has extra "email" field
        
        let table = Table {
            name: "users".to_string(),
            declared_rows: 2,
            rows: vec![row1, row2],
        };
        
        let result = Validator::validate(&Value::Table(table));
        assert!(result.is_err());
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(error_msg.contains("different schema"));
        }
    }

    #[test]
    fn test_check_structure_empty() {
        let obj = Value::Object(HashMap::new());
        let issues = Validator::check_structure(&obj);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("Empty object"));
        
        let arr = Value::Array(vec![]);
        let issues = Validator::check_structure(&arr);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("Empty array"));
    }

    #[test]
    fn test_check_structure_nested() {
        let mut inner = HashMap::new();
        inner.insert("empty".to_string(), Value::Array(vec![]));
        
        let mut obj = HashMap::new();
        obj.insert("nested".to_string(), Value::Object(inner));
        
        let value = Value::Object(obj);
        let issues = Validator::check_structure(&value);
        
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("nested.empty"));
        assert!(issues[0].contains("Empty array"));
    }

    #[test]
    fn test_check_structure_table_empty() {
        let table = Table {
            name: "users".to_string(),
            declared_rows: 5,
            rows: vec![],
        };
        
        let mut obj = HashMap::new();
        obj.insert("users".to_string(), Value::Table(table));
        
        let value = Value::Object(obj);
        let issues = Validator::check_structure(&value);
        
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("declared with 5 rows but is empty"));
    }
}