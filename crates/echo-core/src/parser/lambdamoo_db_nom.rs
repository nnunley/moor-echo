//! LambdaMOO Database Parser using nom
//! 
//! This parser handles LambdaMOO text dump format (version 4+)
//! 
//! # EBNF Grammar
//! 
//! ```ebnf
//! database = header, newline, intro_block, player_list, objects, [verb_programs], [tasks], [clocks];
//! 
//! header = "** LambdaMOO Database, Format Version ", number, " **";
//! 
//! intro_block = total_objects, newline, total_verbs, newline, "0", newline, total_players, newline;
//! total_objects = number;
//! total_verbs = number;
//! total_players = number;
//! 
//! player_list = {player_id, newline};
//! player_id = object_id;
//! 
//! objects = [object_count, newline], {object_def};
//! object_count = number;
//! 
//! object_def = "#", object_id, newline, (object_body | "recycled", newline);
//! object_id = number;
//! 
//! object_body = object_name, newline,
//!               object_handles, newline,  (* usually empty *)
//!               object_flags, newline,
//!               object_owner, newline,
//!               object_location, newline,
//!               object_contents, newline,
//!               object_next, newline,
//!               object_parent, newline,
//!               object_child, newline,
//!               object_sibling, newline,
//!               verb_definitions,
//!               property_definitions,
//!               property_values;
//! 
//! (* Verbs - COUNT BASED *)
//! verb_definitions = verb_count, newline, {verb_def};
//! verb_count = number;
//! verb_def = verb_name, newline, verb_owner, newline, verb_perms, newline, verb_prep, newline;
//! 
//! (* Properties - COUNT BASED *)
//! property_definitions = propdef_count, newline, {propdef};
//! propdef_count = number;
//! propdef = prop_name, newline;
//! 
//! (* Property Values - COUNT BASED *)
//! property_values = propval_count, newline, {propval};
//! propval_count = number;
//! propval = value, prop_owner, newline, prop_perms, newline;
//! 
//! (* Values - TYPE BASED with recursive structure *)
//! value = value_type, newline, value_content;
//! value_type = number;  (* 0=INT, 1=OBJ, 2=STR, 3=ERR, 4=LIST, 5=CLEAR, 6=NONE, 7=CATCH, 8=FINALLY, 9=FLOAT, 12=MAP *)
//! 
//! value_content = simple_value | list_value | map_value;
//! simple_value = line;  (* For INT, OBJ, STR, ERR, FLOAT - just the value on a line *)
//! 
//! (* LIST - LENGTH BASED with recursive values *)
//! list_value = list_length, newline, {value};  (* exactly list_length values *)
//! list_length = number;
//! 
//! (* MAP - LENGTH BASED with recursive key-value pairs *)
//! map_value = map_length, newline, {value, value};  (* exactly map_length pairs *)
//! map_length = number;
//! 
//! (* Verb Programs *)
//! verb_programs = [total_verb_count, newline], {verb_program};
//! verb_program = verb_header, newline, program_code;
//! verb_header = "#", object_id, ":", verb_name;
//! program_code = {program_line}, ".", newline;
//! program_line = line, newline;  (* any line except "." *)
//! 
//! (* Basic types *)
//! number = ["-"], digit, {digit};
//! line = {any_char_except_newline};
//! newline = "\n" | "\r\n";
//! ```

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
use super::moo_common::*;

// Re-export common types for convenience
pub use super::moo_common::{DatabaseVersion, ObjectId, MooValue, MooObject, MooVerb, MooProperty, MooPropertyValue};

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

/// Parse a float number
fn parse_float(input: &str) -> IResult<&str, f64> {
    map_res(
        recognize(tuple((
            opt(char('-')),
            digit1,
            opt(tuple((
                char('.'),
                digit1
            ))),
            opt(tuple((
                alt((char('e'), char('E'))),
                opt(alt((char('+'), char('-')))),
                digit1
            )))
        ))),
        |s: &str| s.parse::<f64>()
    )(input)
}

/// Parse a line of text (everything until newline)
fn parse_line(input: &str) -> IResult<&str, &str> {
    not_line_ending(input)
}

/// Parse a semantic version (e.g., "1.8.1")
fn parse_semantic_version(input: &str) -> IResult<&str, (u32, u32, u32)> {
    tuple((
        map_res(digit1, |s: &str| s.parse::<u32>()),
        preceded(char('.'), map_res(digit1, |s: &str| s.parse::<u32>())),
        preceded(char('.'), map_res(digit1, |s: &str| s.parse::<u32>())),
    ))(input)
}

/// Parse the database version
fn parse_version(input: &str) -> DatabaseVersion {
    // Try simple numeric version first
    if let Ok(num) = input.trim().parse::<i32>() {
        return DatabaseVersion::Numeric(num);
    }
    
    // Try dialect-semantic version (e.g., "LambdaMOO-1.8.1")
    if let Some(dash_pos) = input.find('-') {
        let (dialect, version_part) = input.split_at(dash_pos);
        let version_part = &version_part[1..]; // Skip the dash
        
        if let Ok((_, (major, minor, patch))) = parse_semantic_version(version_part) {
            return DatabaseVersion::Dialect {
                name: dialect.to_string(),
                major,
                minor,
                patch,
            };
        }
    }
    
    // Fall back to arbitrary string
    DatabaseVersion::Other(input.trim().to_string())
}

/// Parse the database header
fn parse_header(input: &str) -> IResult<&str, DatabaseVersion> {
    let (input, _) = tag("** LambdaMOO Database, Format Version ")(input)?;
    let (input, version_str) = take_until(" **")(input)?;
    let (input, _) = tag(" **")(input)?;
    let (input, _) = line_ending(input)?;
    Ok((input, parse_version(version_str)))
}

/// Parse the intro block (4 numbers)
fn parse_intro_block(input: &str) -> IResult<&str, (i64, i64, i64, i64)> {
    tuple((
        terminated(parse_number, line_ending), // total_objects
        terminated(parse_number, line_ending), // total_verbs
        terminated(parse_number, line_ending), // dummy zero
        terminated(parse_number, line_ending), // total_players
    ))(input)
}

/// Parse player list
fn parse_player_list(input: &str) -> IResult<&str, Vec<i64>> {
    many0(terminated(parse_number, line_ending))(input)
}

/// Parse object header (#123)
fn parse_object_header(input: &str) -> IResult<&str, i64> {
    delimited(
        char('#'),
        parse_number,
        line_ending
    )(input)
}

/// Parse a value based on its type
/// This is the KEY function that handles the TYPE-BASED parsing
fn parse_value(input: &str) -> IResult<&str, MooValue> {
    let (input, value_type) = terminated(parse_number, line_ending)(input)?;
    
    match value_type {
        TYPE_CLEAR => Ok((input, MooValue::Clear)),
        TYPE_NONE => Ok((input, MooValue::None)),
        TYPE_INT | TYPE_CATCH | TYPE_FINALLY => {
            let (input, n) = terminated(parse_number, line_ending)(input)?;
            Ok((input, MooValue::Int(n)))
        },
        TYPE_OBJ => {
            let (input, id) = terminated(parse_number, line_ending)(input)?;
            Ok((input, MooValue::Obj(id)))
        },
        TYPE_STR => {
            let (input, s) = terminated(parse_line, line_ending)(input)?;
            Ok((input, MooValue::Str(s.to_string())))
        },
        TYPE_ERR => {
            let (input, code) = terminated(parse_number, line_ending)(input)?;
            Ok((input, MooValue::Err(code)))
        },
        TYPE_FLOAT => {
            let (input, f) = terminated(parse_float, line_ending)(input)?;
            Ok((input, MooValue::Float(f)))
        },
        TYPE_LIST => parse_list_value(input),
        TYPE_MAP => parse_map_value(input),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Switch
        )))
    }
}

/// Parse a list value (LENGTH-BASED with recursive values)
fn parse_list_value(input: &str) -> IResult<&str, MooValue> {
    let (input, length) = terminated(parse_number, line_ending)(input)?;
    let (input, values) = count(parse_value, length as usize)(input)?;
    Ok((input, MooValue::List(values)))
}

/// Parse a map value (LENGTH-BASED with recursive key-value pairs)
fn parse_map_value(input: &str) -> IResult<&str, MooValue> {
    let (input, length) = terminated(parse_number, line_ending)(input)?;
    let (input, pairs) = count(
        tuple((parse_value, parse_value)),
        length as usize
    )(input)?;
    Ok((input, MooValue::Map(pairs)))
}

/// Parse property value (value + owner + perms)
fn parse_property_value(input: &str) -> IResult<&str, MooPropertyValue> {
    let (input, value) = parse_value(input)?;
    let (input, owner) = terminated(parse_number, line_ending)(input)?;
    let (input, perms) = terminated(parse_number, line_ending)(input)?;
    
    Ok((input, MooPropertyValue {
        value,
        owner,
        perms,
    }))
}

/// Parse property values section (COUNT-BASED)
fn parse_property_values(input: &str) -> IResult<&str, Vec<MooPropertyValue>> {
    let (input, num_values) = terminated(parse_number, line_ending)(input)?;
    count(parse_property_value, num_values as usize)(input)
}

/// Parse verb definition
fn parse_verb_def(input: &str) -> IResult<&str, MooVerb> {
    let (input, name) = terminated(parse_line, line_ending)(input)?;
    let (input, owner) = terminated(parse_number, line_ending)(input)?;
    let (input, perms) = terminated(parse_number, line_ending)(input)?;
    let (input, prep) = terminated(parse_number, line_ending)(input)?;
    
    Ok((input, MooVerb {
        name: name.to_string(),
        owner,
        perms,
        prep,
    }))
}

/// Parse verb definitions section (COUNT-BASED)
fn parse_verb_definitions(input: &str) -> IResult<&str, Vec<MooVerb>> {
    let (input, num_verbs) = terminated(parse_number, line_ending)(input)?;
    count(parse_verb_def, num_verbs as usize)(input)
}

/// Parse property definition (just a name)
fn parse_property_def(input: &str) -> IResult<&str, MooProperty> {
    let (input, name) = terminated(parse_line, line_ending)(input)?;
    Ok((input, MooProperty {
        name: name.to_string(),
    }))
}

/// Parse property definitions section (COUNT-BASED)
fn parse_property_definitions(input: &str) -> IResult<&str, Vec<MooProperty>> {
    let (input, num_props) = terminated(parse_number, line_ending)(input)?;
    count(parse_property_def, num_props as usize)(input)
}

/// Parse an object body
fn parse_object_body(input: &str) -> IResult<&str, (MooObject, Vec<MooPropertyValue>)> {
    let (input, name) = terminated(parse_line, line_ending)(input)?;
    let (input, _handles) = terminated(parse_line, line_ending)(input)?; // Usually empty
    let (input, flags) = terminated(parse_number, line_ending)(input)?;
    let (input, owner) = terminated(parse_number, line_ending)(input)?;
    let (input, location) = terminated(parse_number, line_ending)(input)?;
    let (input, contents) = terminated(parse_number, line_ending)(input)?;
    let (input, next) = terminated(parse_number, line_ending)(input)?;
    let (input, parent) = terminated(parse_number, line_ending)(input)?;
    let (input, child) = terminated(parse_number, line_ending)(input)?;
    let (input, sibling) = terminated(parse_number, line_ending)(input)?;
    let (input, verbs) = parse_verb_definitions(input)?;
    let (input, properties) = parse_property_definitions(input)?;
    let (input, property_values) = parse_property_values(input)?;
    
    let object = MooObject {
        id: 0, // Will be set by caller
        name: name.to_string(),
        flags,
        owner,
        location,
        contents,
        next,
        parent,
        child,
        sibling,
        verbs,
        properties,
        property_values: vec![], // Will be filled by caller
        is_recycled: false,
    };
    
    Ok((input, (object, property_values)))
}

/// Parse an object definition
fn parse_object_def(input: &str) -> IResult<&str, Option<(MooObject, Vec<MooPropertyValue>)>> {
    let (input, id) = parse_object_header(input)?;
    
    // Check for recycled object
    if let Ok((input, _)) = tuple::<_, _, nom::error::Error<_>, _>((tag("recycled"), line_ending))(input) {
        return Ok((input, None));
    }
    
    // Parse object body
    match parse_object_body(input) {
        Ok((input, (mut obj, prop_values))) => {
            obj.id = id;
            Ok((input, Some((obj, prop_values))))
        }
        Err(e) => {
            // Debug output to help diagnose the issue
            if id == 0 {
                eprintln!("DEBUG: Failed to parse object #0 body: {:?}", e);
            }
            Err(e)
        }
    }
}

/// Parse the entire object list
fn parse_objects(input: &str) -> IResult<&str, HashMap<i64, MooObject>> {
    // Optional object count
    let (input, _count) = opt(terminated(parse_number, line_ending))(input)?;
    
    // Parse all objects
    let (input, object_defs) = many0(parse_object_def)(input)?;
    
    let mut objects = HashMap::new();
    for def in object_defs {
        if let Some((mut obj, prop_values)) = def {
            obj.property_values = prop_values;
            objects.insert(obj.id, obj);
        }
    }
    
    Ok((input, objects))
}

/// Parse a single program line (everything until newline, but not a single ".")
fn parse_program_line(input: &str) -> IResult<&str, &str> {
    // Check if the line is just "." which terminates the program
    if input.starts_with(".\n") || input.starts_with(".\r\n") {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag
        )));
    }
    
    terminated(not_line_ending, line_ending)(input)
}

/// Parse program code (multiple lines until "." on its own line)
fn parse_program_code(input: &str) -> IResult<&str, String> {
    let (input, lines) = many0(parse_program_line)(input)?;
    let (input, _) = tuple((tag("."), line_ending))(input)?;
    
    Ok((input, lines.join("\n")))
}

/// Parse verb header (e.g., "#123:verb_name")
fn parse_verb_header(input: &str) -> IResult<&str, (ObjectId, String)> {
    let (input, _) = char('#')(input)?;
    let (input, obj_id) = parse_number(input)?;
    let (input, _) = char(':')(input)?;
    let (input, verb_name) = terminated(not_line_ending, line_ending)(input)?;
    
    Ok((input, (obj_id, verb_name.to_string())))
}

/// Parse a single verb program
fn parse_verb_program(input: &str) -> IResult<&str, ((ObjectId, String), String)> {
    let (input, (obj_id, verb_name)) = parse_verb_header(input)?;
    let (input, code) = parse_program_code(input)?;
    
    Ok((input, ((obj_id, verb_name), code)))
}

/// Parse the verbs section
fn parse_verbs_section(input: &str) -> IResult<&str, HashMap<(ObjectId, String), String>> {
    // Optional total verb count (some databases have it, some don't)
    let (input, _count) = opt(terminated(parse_number, line_ending))(input)?;
    
    // Parse all verb programs
    let (input, programs) = many0(parse_verb_program)(input)?;
    
    let mut verb_map = HashMap::new();
    for (key, code) in programs {
        verb_map.insert(key, code);
    }
    
    Ok((input, verb_map))
}

/// Parse ToastStunt extensions for format 17
fn parse_toaststunt_extensions(input: &str) -> IResult<&str, ()> {
    // ToastStunt format 17 has numbers after the version that aren't part of standard intro
    // Let's consume them without interpreting
    let (input, _) = terminated(parse_number, line_ending)(input)?; // 71
    let (input, _) = terminated(parse_number, line_ending)(input)?; // 36
    let (input, _) = terminated(parse_number, line_ending)(input)?; // 38
    let (input, _) = terminated(parse_number, line_ending)(input)?; // 96
    let (input, _) = terminated(parse_number, line_ending)(input)?; // 98
    
    // Parse "X values pending finalization"
    let (input, _pending_count) = terminated(parse_number, tuple((tag(" values pending finalization"), line_ending)))(input)?;
    
    // Parse "X clocks"  
    let (input, _clocks_count) = terminated(parse_number, tuple((tag(" clocks"), line_ending)))(input)?;
    
    // Parse "X queued tasks"
    let (input, _task_count) = terminated(parse_number, tuple((tag(" queued tasks"), line_ending)))(input)?;
    
    // Skip the complex task/extension data. Look for the pattern that precedes the object count:
    // "X active connections with listeners" followed by the object count and then "#0"
    let (input, _) = take_until("active connections with listeners")(input)?;
    let (input, _) = take_until("\n")(input)?; // Skip to end of connections line
    let (input, _) = line_ending(input)?; // Consume the newline
    
    // Now we should be positioned at the object count line (127) before "#0"
    
    Ok((input, ()))
}

// Re-export the common MooDatabase type
pub use super::moo_common::MooDatabase;

/// Parse a complete MOO database
pub fn parse_database(input: &str) -> Result<MooDatabase> {
    let (input, version) = parse_header(input)
        .map_err(|e| anyhow!("Failed to parse header: {:?}", e))?;
    
    // Skip any extra newlines between sections
    let (input, _) = many0::<_, _, nom::error::Error<_>, _>(line_ending)(input)
        .map_err(|e| anyhow!("Failed to skip newlines after header: {:?}", e))?;
    
    // Try to detect if this is a ToastStunt database by checking version and structure
    let is_toaststunt = version == DatabaseVersion::Numeric(17) || input.contains("values pending finalization");
    
    let (input, total_objects, total_verbs, total_players, players, objects, verb_programs) = if is_toaststunt {
        // ToastStunt format 17 has a different structure
        // The intro block format is different - need to handle it specially
        let (input, maybe_first_num) = terminated(parse_number, line_ending)(input)
            .map_err(|e| anyhow!("Failed to parse first number in ToastStunt intro: {:?}", e))?;
        let (input, maybe_second_num) = terminated(parse_number, line_ending)(input)
            .map_err(|e| anyhow!("Failed to parse second number in ToastStunt intro: {:?}", e))?;
        
        // For now, treat these as total_objects and total_verbs (need to verify)
        let total_objects = maybe_first_num;
        let total_verbs = maybe_second_num;
        let total_players = 0; // ToastStunt might not have this in the same place
        
        // Parse ToastStunt extensions
        let (input, _) = parse_toaststunt_extensions(input)
            .map_err(|e| anyhow!("Failed to parse ToastStunt extensions: {:?}", e))?;
        
        // For now, parse objects without worrying about the exact counts
        let (input, objects) = parse_objects(input)
            .map_err(|e| anyhow!("Failed to parse ToastStunt objects: {:?}", e))?;
        
        // Parse verb programs (ToastStunt format 17 also has verb programs after objects)
        let (_input, verb_programs) = parse_verbs_section(input)
            .unwrap_or((input, HashMap::new()));
        let players = vec![];
        
        (input, total_objects, total_verbs, total_players, players, objects, verb_programs)
    } else {
        // Standard LambdaMOO format
        let (input, (total_objects, total_verbs, _, total_players)) = parse_intro_block(input)
            .map_err(|e| anyhow!("Failed to parse intro block: {:?}", e))?;
        
        // Parse exactly the number of players specified
        let (input, players) = count(terminated(parse_number, line_ending), total_players as usize)(input)
            .map_err(|e| anyhow!("Failed to parse player list: {:?}", e))?;
        
        // Skip any extra newlines after player list
        let (input, _) = many0::<_, _, nom::error::Error<_>, _>(line_ending)(input)
            .map_err(|e| anyhow!("Failed to skip newlines after player list: {:?}", e))?;
        
        // Parse objects first, then verb programs
        let (input, objects) = parse_objects(input)
            .map_err(|e| anyhow!("Failed to parse objects: {:?}", e))?;
        
        // Parse verb programs (optional - might not be present in all databases)
        let (_input, verb_programs) = parse_verbs_section(input)
            .unwrap_or((input, HashMap::new()));
        
        (input, total_objects, total_verbs, total_players, players, objects, verb_programs)
    };
    
    // TODO: Parse tasks, clocks, etc.
    
    Ok(MooDatabase {
        name: String::new(), // Will be set by caller
        path: String::new(), // Will be set by caller
        version,
        total_objects,
        total_verbs,
        total_players,
        players,
        objects,
        verb_programs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_number("42"), Ok(("", 42)));
        assert_eq!(parse_number("-42"), Ok(("", -42)));
        assert_eq!(parse_number("0"), Ok(("", 0)));
    }

    #[test]
    fn test_parse_value_int() {
        assert_eq!(parse_value("0\n42\n"), Ok(("", MooValue::Int(42))));
    }

    #[test]
    fn test_parse_value_string() {
        assert_eq!(parse_value("2\nhello world\n"), Ok(("", MooValue::Str("hello world".to_string()))));
    }

    #[test]
    fn test_parse_value_list() {
        let input = "4\n2\n0\n42\n0\n99\n";
        assert_eq!(
            parse_value(input),
            Ok(("", MooValue::List(vec![
                MooValue::Int(42),
                MooValue::Int(99)
            ])))
        );
    }

    #[test]
    fn test_parse_value_nested_list() {
        let input = "4\n2\n0\n42\n4\n2\n0\n1\n0\n2\n";
        assert_eq!(
            parse_value(input),
            Ok(("", MooValue::List(vec![
                MooValue::Int(42),
                MooValue::List(vec![
                    MooValue::Int(1),
                    MooValue::Int(2)
                ])
            ])))
        );
    }

    #[test]
    fn test_parse_property_value_with_list() {
        let input = "4\n2\n2\ngenerics\n2\nGeneric objects intended for use as the parents of new objects\n4\n3\n";
        let result = parse_property_value(input);
        assert!(result.is_ok());
        let (_, propval) = result.unwrap();
        assert_eq!(propval.owner, 4);
        assert_eq!(propval.perms, 3);
        match propval.value {
            MooValue::List(ref items) => {
                assert_eq!(items.len(), 2);
                match &items[0] {
                    MooValue::Str(s) => assert_eq!(s, "generics"),
                    _ => panic!("Expected string in list"),
                }
            }
            _ => panic!("Expected list value"),
        }
    }
    
    #[test]
    fn test_parse_program_line() {
        assert_eq!(parse_program_line("hello world\n"), Ok(("", "hello world")));
        assert_eq!(parse_program_line("return 42;\n"), Ok(("", "return 42;")));
        assert_eq!(parse_program_line("\n"), Ok(("", "")));
        
        // Should fail on program terminator
        assert!(parse_program_line(".\n").is_err());
    }
    
    #[test]
    fn test_parse_program_code() {
        let input = "line 1\nline 2\nline 3\n.\n";
        assert_eq!(
            parse_program_code(input),
            Ok(("", "line 1\nline 2\nline 3".to_string()))
        );
        
        // Empty program
        let input = ".\n";
        assert_eq!(
            parse_program_code(input),
            Ok(("", "".to_string()))
        );
    }
    
    #[test]
    fn test_parse_verb_header() {
        assert_eq!(
            parse_verb_header("#123:test_verb\n"),
            Ok(("", (123, "test_verb".to_string())))
        );
        
        assert_eq!(
            parse_verb_header("#0:server_started\n"),
            Ok(("", (0, "server_started".to_string())))
        );
    }
    
    #[test]
    fn test_parse_verb_program() {
        let input = "#123:test_verb\nline 1\nline 2\n.\n";
        let (remaining, ((obj_id, verb_name), code)) = parse_verb_program(input).unwrap();
        
        assert_eq!(remaining, "");
        assert_eq!(obj_id, 123);
        assert_eq!(verb_name, "test_verb");
        assert_eq!(code, "line 1\nline 2");
    }
    
    #[test]
    fn test_parse_verbs_section() {
        let input = "2\n#1:test1\ncode1\n.\n#2:test2\ncode2\nline2\n.\n";
        let (remaining, programs) = parse_verbs_section(input).unwrap();
        
        assert_eq!(remaining, "");
        assert_eq!(programs.len(), 2);
        assert_eq!(programs.get(&(1, "test1".to_string())), Some(&"code1".to_string()));
        assert_eq!(programs.get(&(2, "test2".to_string())), Some(&"code2\nline2".to_string()));
    }
}