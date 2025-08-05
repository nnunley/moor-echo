use std::fs;
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/lambdamoo_db_grammar_simple.pest"]
pub struct SimpleLambdaMooParser;

#[derive(Debug)]
pub struct SimpleObject {
    pub id: i64,
    pub name: String,
}

fn parse_simple_database(input: &str) -> Result<Vec<SimpleObject>> {
    let pairs = SimpleLambdaMooParser::parse(Rule::database, input)
        .map_err(|e| anyhow!("Parse error: {}", e))?;
    
    let mut objects = Vec::new();
    
    for pair in pairs {
        if pair.as_rule() == Rule::database {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::content {
                    for content_item in inner.into_inner() {
                        if content_item.as_rule() == Rule::rest_of_database {
                            for db_line in content_item.into_inner() {
                                if db_line.as_rule() == Rule::database_line {
                                    for line_item in db_line.into_inner() {
                                        if line_item.as_rule() == Rule::object_def {
                                            if let Some(obj) = parse_object_def(line_item)? {
                                                objects.push(obj);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(objects)
}

fn parse_object_def(pair: pest::iterators::Pair<Rule>) -> Result<Option<SimpleObject>> {
    let mut obj_id = None;
    let mut obj_name = None;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::object_header => {
                for header_item in inner.into_inner() {
                    if header_item.as_rule() == Rule::objid {
                        for id_item in header_item.into_inner() {
                            if id_item.as_rule() == Rule::num {
                                obj_id = Some(id_item.as_str().parse::<i64>()?);
                            }
                        }
                    }
                }
            }
            Rule::object_content => {
                // Get the first line as the name
                let mut lines = inner.into_inner();
                if let Some(first_line) = lines.next() {
                    if first_line.as_rule() == Rule::object_line {
                        for line_item in first_line.into_inner() {
                            if line_item.as_rule() == Rule::raw_string {
                                obj_name = Some(line_item.as_str().to_string());
                                break;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    
    if let (Some(id), Some(name)) = (obj_id, obj_name) {
        Ok(Some(SimpleObject { id, name }))
    } else {
        Ok(None)
    }
}

fn main() -> Result<()> {
    let databases = vec![
        "examples/Minimal.db",
        "examples/LambdaCore-latest.db",
        "examples/toastcore.db",
        "examples/JHCore-DEV-2.db",
    ];
    
    for db_path in databases {
        if std::path::Path::new(db_path).exists() {
            println!("\n=== Testing {} ===", db_path);
            let content = fs::read_to_string(db_path)?;
            
            match parse_simple_database(&content) {
                Ok(objects) => {
                    println!("Successfully parsed {} objects", objects.len());
                    
                    // Look for object #1
                    if let Some(obj1) = objects.iter().find(|obj| obj.id == 1) {
                        println!("✓ Object #1 found: {}", obj1.name);
                    } else {
                        println!("✗ Object #1 not found");
                    }
                    
                    // Show first few objects
                    println!("First 5 objects:");
                    for obj in objects.iter().take(5) {
                        println!("  #{}: {}", obj.id, obj.name);
                    }
                }
                Err(e) => {
                    println!("Failed to parse: {}", e);
                }
            }
        } else {
            println!("Database {} not found", db_path);
        }
    }
    
    Ok(())
}