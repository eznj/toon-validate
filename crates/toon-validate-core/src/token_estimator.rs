use crate::value::Value;

pub struct TokenEstimator;

impl TokenEstimator {
    pub fn estimate(value: &Value) -> usize {
        let chars = Self::count_characters(value);
        // Simple heuristic: approximately 4 characters per token
        (chars + 3) / 4
    }
    
    pub fn estimate_breakdown(value: &Value) -> TokenBreakdown {
        let mut breakdown = TokenBreakdown::new();
        Self::analyze_value(value, &mut breakdown);
        breakdown
    }
    
    fn count_characters(value: &Value) -> usize {
        match value {
            Value::Null => 4,
            Value::Bool(b) => if *b { 4 } else { 5 },
            Value::Number(n) => n.to_string().len(),
            Value::String(s) => s.len() + 2, // Include quotes
            Value::Array(arr) => {
                let mut total = 2; // []
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        total += 2; // ", "
                    }
                    total += Self::count_characters(item);
                }
                total
            }
            Value::Object(obj) => {
                let mut total = 2; // {}
                for (i, (key, val)) in obj.iter().enumerate() {
                    if i > 0 {
                        total += 2; // ", "
                    }
                    total += key.len() + 3; // "key": 
                    total += Self::count_characters(val);
                }
                total
            }
            Value::Table(table) => {
                let mut total = table.name.len() + 10; // name[N]:
                for row in &table.rows {
                    total += 2; // "- "
                    for (key, val) in row {
                        total += key.len() + 2; // key: 
                        total += Self::count_characters(val);
                    }
                }
                total
            }
        }
    }
    
    fn analyze_value(value: &Value, breakdown: &mut TokenBreakdown) {
        match value {
            Value::Null => breakdown.add_primitive(1),
            Value::Bool(_) => breakdown.add_primitive(1),
            Value::Number(_) => breakdown.add_primitive(1),
            Value::String(s) => breakdown.add_string((s.len() + 3) / 4),
            Value::Array(arr) => {
                breakdown.add_structure(1);
                for item in arr {
                    Self::analyze_value(item, breakdown);
                }
            }
            Value::Object(obj) => {
                breakdown.add_structure(obj.len());
                for (key, val) in obj {
                    breakdown.add_key((key.len() + 3) / 4);
                    Self::analyze_value(val, breakdown);
                }
            }
            Value::Table(table) => {
                breakdown.add_table(
                    (table.name.len() + 3) / 4,
                    table.rows.len()
                );
                for row in &table.rows {
                    for (key, val) in row {
                        breakdown.add_key((key.len() + 3) / 4);
                        Self::analyze_value(val, breakdown);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenBreakdown {
    pub keys: usize,
    pub strings: usize,
    pub primitives: usize,
    pub structure: usize,
    pub tables: usize,
    pub table_rows: usize,
}

impl TokenBreakdown {
    pub fn new() -> Self {
        TokenBreakdown {
            keys: 0,
            strings: 0,
            primitives: 0,
            structure: 0,
            tables: 0,
            table_rows: 0,
        }
    }
    
    pub fn add_key(&mut self, tokens: usize) {
        self.keys += tokens;
    }
    
    pub fn add_string(&mut self, tokens: usize) {
        self.strings += tokens;
    }
    
    pub fn add_primitive(&mut self, tokens: usize) {
        self.primitives += tokens;
    }
    
    pub fn add_structure(&mut self, tokens: usize) {
        self.structure += tokens;
    }
    
    pub fn add_table(&mut self, name_tokens: usize, rows: usize) {
        self.tables += name_tokens;
        self.table_rows += rows;
    }
    
    pub fn total(&self) -> usize {
        self.keys + self.strings + self.primitives + self.structure + self.tables
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Table;
    use std::collections::HashMap;

    #[test]
    fn test_estimate_primitives() {
        assert_eq!(TokenEstimator::estimate(&Value::Null), 1);
        assert_eq!(TokenEstimator::estimate(&Value::Bool(true)), 1);
        assert_eq!(TokenEstimator::estimate(&Value::Bool(false)), 2);
        assert_eq!(TokenEstimator::estimate(&Value::Number(42.0)), 1);
        assert_eq!(TokenEstimator::estimate(&Value::Number(12345.0)), 2);
    }

    #[test]
    fn test_estimate_string() {
        assert_eq!(TokenEstimator::estimate(&Value::String("test".to_string())), 2);
        assert_eq!(TokenEstimator::estimate(&Value::String("a longer string".to_string())), 5);
    }

    #[test]
    fn test_estimate_array() {
        let arr = Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        // [1, 2, 3] = 8 chars + 2 for brackets = 10 chars / 4 ≈ 3 tokens
        assert_eq!(TokenEstimator::estimate(&arr), 3);
    }

    #[test]
    fn test_estimate_object() {
        let mut obj = HashMap::new();
        obj.insert("name".to_string(), Value::String("Alice".to_string()));
        obj.insert("age".to_string(), Value::Number(30.0));
        let value = Value::Object(obj);
        
        // Estimate should be reasonable for object
        let estimate = TokenEstimator::estimate(&value);
        assert!(estimate > 0);
        assert!(estimate < 20); // Should be reasonable for small object
    }

    #[test]
    fn test_breakdown_simple() {
        let mut obj = HashMap::new();
        obj.insert("name".to_string(), Value::String("test".to_string()));
        obj.insert("count".to_string(), Value::Number(5.0));
        obj.insert("active".to_string(), Value::Bool(true));
        let value = Value::Object(obj);
        
        let breakdown = TokenEstimator::estimate_breakdown(&value);
        
        assert_eq!(breakdown.keys, 5); // 3 keys with total token count of 5
        assert_eq!(breakdown.strings, 1); // 1 string value ("test") = 1 token
        assert_eq!(breakdown.primitives, 2); // 1 number + 1 bool
        assert_eq!(breakdown.structure, 3); // object structure
        assert_eq!(breakdown.tables, 0);
        assert_eq!(breakdown.table_rows, 0);
    }

    #[test]
    fn test_breakdown_with_table() {
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
        
        let mut obj = HashMap::new();
        obj.insert("users".to_string(), Value::Table(table));
        let value = Value::Object(obj);
        
        let breakdown = TokenEstimator::estimate_breakdown(&value);
        
        assert!(breakdown.keys > 0); // table row keys
        assert!(breakdown.strings > 0); // string values in table
        assert!(breakdown.primitives > 0); // number values in table
        assert_eq!(breakdown.tables, 2); // table name tokens
        assert_eq!(breakdown.table_rows, 2); // 2 rows
    }

    #[test]
    fn test_breakdown_total() {
        let breakdown = TokenBreakdown {
            keys: 10,
            strings: 5,
            primitives: 3,
            structure: 2,
            tables: 1,
            table_rows: 0,
        };
        
        assert_eq!(breakdown.total(), 21);
    }
}