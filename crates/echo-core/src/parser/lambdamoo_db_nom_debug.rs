//! Debug version of the parser with logging

use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char, digit1, line_ending, not_line_ending},
    combinator::{map_res, opt, recognize},
    multi::{count, many0},
    sequence::{delimited, preceded, terminated, tuple},
};
use std::collections::HashMap;
use anyhow::{Result, anyhow};

// Import common types
use super::moo_common::*;

/// Parse a number (positive or negative integer)
fn parse_number(input: &str) -> IResult<&str, i64> {
    map_res(
        recognize(tuple((
            opt(char('-')),
            digit1
        ))),
        |s: &str| s.parse::<i64>()
    )(input)
}

/// Parse object header (#123)
fn parse_object_header(input: &str) -> IResult<&str, i64> {
    delimited(
        char('#'),
        parse_number,
        line_ending
    )(input)
}

/// Parse an object definition with debug output
fn parse_object_def_debug(input: &str) -> IResult<&str, Option<(i64, String)>> {
    eprintln!("DEBUG: parse_object_def_debug called with input starting: {:?}", &input[..50.min(input.len())]);
    
    let (input, id) = parse_object_header(input)?;
    eprintln!("DEBUG: Parsed object header: #{}", id);
    
    // For now, just skip to the next object or end
    let mut remaining = input;
    let mut line_count = 0;
    loop {
        if remaining.is_empty() || remaining.starts_with('#') {
            break;
        }
        let next_newline = remaining.find('\n').unwrap_or(remaining.len());
        remaining = &remaining[next_newline..];
        if !remaining.is_empty() {
            remaining = &remaining[1..]; // Skip the newline
        }
        line_count += 1;
    }
    
    eprintln!("DEBUG: Object #{} consumed {} lines", id, line_count);
    Ok((remaining, Some((id, format!("Object #{}", id)))))
}

/// Parse the entire object list with debug
pub fn parse_objects_debug(input: &str) -> IResult<&str, Vec<i64>> {
    eprintln!("DEBUG: parse_objects_debug called");
    eprintln!("DEBUG: Input starts with: {:?}", &input[..100.min(input.len())]);
    
    // Optional object count
    let (input, obj_count) = opt(terminated(parse_number, line_ending))(input)?;
    eprintln!("DEBUG: Optional object count: {:?}", obj_count);
    
    // Parse all objects
    eprintln!("DEBUG: Calling many0(parse_object_def_debug)...");
    let (input, object_defs) = many0(parse_object_def_debug)(input)?;
    eprintln!("DEBUG: many0 returned {} objects", object_defs.len());
    
    let ids: Vec<i64> = object_defs.into_iter()
        .filter_map(|def| def.map(|(id, _)| id))
        .collect();
    
    Ok((input, ids))
}