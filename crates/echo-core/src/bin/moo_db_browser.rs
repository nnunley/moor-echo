use std::collections::HashMap;
use std::fs;
use std::io::stdout;
use std::path::Path;
use std::time::Duration;

use anyhow::{anyhow, Result};
// use tracing::{warn, error};
use std::time::Instant;

// Import the proper LambdaMOO parser and common structures
use echo_core::parser::lambdamoo_db_nom::parse_database;
use echo_core::parser::moo_common::{MooDatabase as CommonMooDatabase, MooObject as CommonMooObject, MooValue, MooVerb as CommonMooVerb, MooProperty as CommonMooProperty, DatabaseVersion};

/// Format object flags as bit flag names
fn format_object_flags(flags: i64) -> String {
    let mut flag_names = Vec::new();

    if flags & 0x01 != 0 { flag_names.push("player".to_string()); }
    if flags & 0x02 != 0 { flag_names.push("programmer".to_string()); }
    if flags & 0x04 != 0 { flag_names.push("wizard".to_string()); }
    if flags & 0x10 != 0 { flag_names.push("read".to_string()); }
    if flags & 0x20 != 0 { flag_names.push("write".to_string()); }
    if flags & 0x80 != 0 { flag_names.push("fertile".to_string()); }

    if flag_names.is_empty() {
        format!("0x{:x}", flags)
    } else {
        format!("{} (0x{:x})", flag_names.join(", "), flags)
    }
}

/// Format object flags as compact single letters
fn format_object_flags_compact(flags: i64) -> String {
    let mut flag_chars = Vec::new();

    if flags & 0x01 != 0 { flag_chars.push('p'); } // player
    if flags & 0x02 != 0 { flag_chars.push('P'); } // programmer
    if flags & 0x04 != 0 { flag_chars.push('w'); } // wizard
    if flags & 0x10 != 0 { flag_chars.push('r'); } // read
    if flags & 0x20 != 0 { flag_chars.push('W'); } // write
    if flags & 0x80 != 0 { flag_chars.push('f'); } // fertile

    if flag_chars.is_empty() {
        format!("0x{:x}", flags)
    } else {
        flag_chars.into_iter().collect()
    }
}

/// Format verb permissions as bit flag names
fn format_verb_permissions(perms: i64) -> String {
    let mut flag_names = Vec::new();

    if perms & 0x01 != 0 { flag_names.push("read".to_string()); }
    if perms & 0x02 != 0 { flag_names.push("write".to_string()); }
    if perms & 0x04 != 0 { flag_names.push("execute".to_string()); }
    if perms & 0x08 != 0 { flag_names.push("debug".to_string()); }

    // dobj arg flags (bits 4-5)
    let dobj_arg = (perms & 0x30) >> 4;
    match dobj_arg {
        0 => flag_names.push("dobj:none".to_string()),
        1 => flag_names.push("dobj:any".to_string()),
        2 => flag_names.push("dobj:this".to_string()),
        _ => flag_names.push(format!("dobj:{}", dobj_arg)),
    }

    // iobj arg flags (bits 6-7)
    let iobj_arg = (perms & 0xC0) >> 6;
    match iobj_arg {
        0 => flag_names.push("iobj:none".to_string()),
        1 => flag_names.push("iobj:any".to_string()),
        2 => flag_names.push("iobj:this".to_string()),
        _ => flag_names.push(format!("iobj:{}", iobj_arg)),
    }

    format!("{} (0x{:x})", flag_names.join(", "), perms)
}

/// Format verb permissions as compact mixed format
fn format_verb_permissions_compact(perms: i64) -> String {
    let mut parts = Vec::new();

    // Basic permission flags as single letters
    let mut basic_flags = String::new();
    if perms & 0x01 != 0 { basic_flags.push('r'); } // read
    if perms & 0x02 != 0 { basic_flags.push('w'); } // write
    if perms & 0x04 != 0 { basic_flags.push('x'); } // execute
    if perms & 0x08 != 0 { basic_flags.push('d'); } // debug

    if !basic_flags.is_empty() {
        parts.push(basic_flags);
    }

    // dobj arg flags (bits 4-5) - keep readable
    let dobj_arg = (perms & 0x30) >> 4;
    match dobj_arg {
        0 => parts.push("none".to_string()),
        1 => parts.push("any".to_string()),
        2 => parts.push("this".to_string()),
        _ => parts.push(format!("dobj:{}", dobj_arg)),
    }

    // iobj arg flags (bits 6-7) - keep readable
    let iobj_arg = (perms & 0xC0) >> 6;
    match iobj_arg {
        0 => parts.push("none".to_string()),
        1 => parts.push("any".to_string()),
        2 => parts.push("this".to_string()),
        _ => parts.push(format!("iobj:{}", iobj_arg)),
    }

    parts.join(" ")
}

/// Format property permissions as bit flag names
fn format_property_permissions(perms: i64) -> String {
    let mut flag_names = Vec::new();

    if perms & 0x01 != 0 { flag_names.push("read".to_string()); }
    if perms & 0x02 != 0 { flag_names.push("write".to_string()); }
    if perms & 0x04 != 0 { flag_names.push("chown".to_string()); }

    if flag_names.is_empty() {
        format!("0x{:x}", perms)
    } else {
        format!("{} (0x{:x})", flag_names.join(", "), perms)
    }
}

/// Format property permissions as compact single letters
fn format_property_permissions_compact(perms: i64) -> String {
    let mut flag_chars = Vec::new();

    if perms & 0x01 != 0 { flag_chars.push('r'); } // read
    if perms & 0x02 != 0 { flag_chars.push('w'); } // write
    if perms & 0x04 != 0 { flag_chars.push('c'); } // chown

    if flag_chars.is_empty() {
        format!("0x{:x}", perms)
    } else {
        flag_chars.into_iter().collect()
    }
}

/// Pretty-print MOO code with proper indentation and syntax highlighting
fn (code: &str) -> String {
    let mut result = Vec::new();
    let mut indent_level: usize = 0;
    let indent = "  "; // 2 spaces per indent level

    for line in code.lines() {
        let trimmed = line.trim();

        // Skip empty lines but preserve them
        if trimmed.is_empty() {
            result.push(String::new());
            continue;
        }

        // Decrease indent for closing keywords
        if trimmed == "endif" || trimmed == "endfor" || trimmed == "endwhile"
            || trimmed == "endtry" || trimmed == "endfork" || trimmed.starts_with("}")
            || trimmed == "except" || trimmed == "finally" {
            indent_level = indent_level.saturating_sub(1);
        }

        // Special handling for else/elseif/except/finally - they're at the same level as their opening
        let line_indent = if trimmed == "else" || trimmed.starts_with("elseif ")
            || trimmed.starts_with("except ") || trimmed == "finally" {
            indent_level.saturating_sub(1)
        } else {
            indent_level
        };

        // Add the line with current indentation
        result.push(format!("{}{}", indent.repeat(line_indent), trimmed));

        // Increase indent after opening keywords
        if trimmed.starts_with("if ") || trimmed == "else" || trimmed.starts_with("elseif ")
            || trimmed.starts_with("for ") || trimmed.starts_with("while ")
            || trimmed.starts_with("try") || trimmed.starts_with("except ")
            || trimmed == "finally" || trimmed.starts_with("fork ")
            || trimmed.ends_with("{") {
            indent_level += 1;
        }
    }

    result.join("\n")
}

/// Format a MooValue for display
fn format_moo_value(value: &MooValue) -> String {
    match value {
        MooValue::Int(n) => n.to_string(),
        MooValue::Obj(id) => format!("#{}", id),
        MooValue::Str(s) => format!("\"{}\"", s),
        MooValue::Err(e) => format!("E_{}", e),
        MooValue::Float(f) => f.to_string(),
        MooValue::Clear => "~clear~".to_string(),
        MooValue::None => "~none~".to_string(),
        MooValue::List(items) => {
            let formatted_items: Vec<String> = items.iter()
                .map(format_moo_value)
                .collect();
            format!("{{{}}}", formatted_items.join(", "))
        }
        MooValue::Map(entries) => {
            let formatted_entries: Vec<String> = entries.iter()
                .map(|(k, v)| format!("{} -> {}", format_moo_value(k), format_moo_value(v)))
                .collect();
            format!("[{}]", formatted_entries.join(", "))
        }
    }
}

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Tabs, Wrap,
    },
    Frame, Terminal,
};

/// Represents a MOO verb with actual code
#[derive(Debug, Clone)]
pub struct MooVerb {
    pub name: String,
    pub owner: i64,
    pub permissions: i64,  // Store raw permissions value
    pub code: String,
}

/// Represents a MOO property with actual value
#[derive(Debug, Clone)]
pub struct MooProperty {
    pub name: String,
    pub value: String,
    pub owner: i64,
    pub permissions: i64,  // Store raw permissions value
}

/// Represents a MOO object with lazy loading
#[derive(Debug, Clone)]
pub enum ObjectState {
    Placeholder,  // Forward reference, not yet parsed
    Parsed(ParsedObjectData),  // Fully parsed object data
    ParseFailed(String),  // Parsing failed with error message
}

#[derive(Debug, Clone)]
pub struct MooObject {
    pub id: i64,
    pub state: ObjectState,
}

#[derive(Debug, Clone)]
pub struct ParsedObjectData {
    pub name: String,
    pub flags: i64,
    pub owner: i64,
    pub location: i64,
    pub parent: i64,
    pub verbs: Vec<MooVerb>,
    pub properties: Vec<MooProperty>,
    pub defined_prop_count: usize,  // Number of properties defined on this object
}

impl MooObject {
    pub fn new_placeholder(id: i64) -> Self {
        Self { id, state: ObjectState::Placeholder }
    }

    pub fn name(&self) -> String {
        match &self.state {
            ObjectState::Placeholder => format!("Object #{} (loading...)", self.id),
            ObjectState::Parsed(data) => data.name.clone(),
            ObjectState::ParseFailed(error) => format!("Object #{} (parse failed: {})", self.id, error),
        }
    }

    pub fn is_parsed(&self) -> bool {
        matches!(self.state, ObjectState::Parsed(_))
    }

    pub fn parsed_data(&self) -> Option<&ParsedObjectData> {
        match &self.state {
            ObjectState::Parsed(data) => Some(data),
            _ => None,
        }
    }
}

/// Represents a MOO database with full parsing
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MooDatabase {
    pub name: String,
    pub path: String,
    pub version: i32,
    pub total_objects: i64,
    pub total_verbs: i64,
    pub total_players: i64,
    pub players: Vec<i64>,
    pub objects: HashMap<i64, MooObject>,
    pub verb_code_map: HashMap<(i64, String), String>,
}

/// MOO database parser
pub struct MooDatabaseParser;

impl MooDatabaseParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse database using the proper LambdaMOO parser
    pub fn parse_database_proper(&self, path: &str, name: &str) -> Result<MooDatabase> {
        let parse_start = Instant::now();
        // Starting to parse database with nom parser

        // Read file
        let file_read_start = Instant::now();
        let content = fs::read_to_string(path)?;
        // info!("Read file {} in {:?}, {} lines", path, file_read_start.elapsed(), content.lines().count());

        // Use the nom parser
        let mut lambda_db = parse_database(&content)?;
        lambda_db.name = name.to_string();
        lambda_db.path = path.to_string();
        // info!("Parsed {} objects with nom parser in {:?}", lambda_db.objects.len(), parse_start.elapsed());

        // Convert to our internal format
        let mut objects = HashMap::new();
        let mut all_object_ids = Vec::new();

        for (obj_id, lambda_obj) in &lambda_db.objects {
            all_object_ids.push(*obj_id);

            // Convert verbs
            let verbs: Vec<MooVerb> = lambda_obj.verbs.iter().enumerate().map(|(verb_index, v)| {
                let code = lambda_db.verb_programs
                    .get(&(*obj_id, verb_index.to_string()))
                    .cloned()
                    .unwrap_or_default();
                MooVerb {
                    name: v.name.clone(),
                    owner: v.owner,
                    permissions: v.perms,
                    code,
                }
            }).collect();

            // Convert properties - match property definitions with values
            let properties: Vec<MooProperty> = lambda_obj.properties.iter().enumerate().map(|(idx, prop)| {
                let prop_value = lambda_obj.property_values.get(idx)
                    .map(|pv| format_moo_value(&pv.value))
                    .unwrap_or_else(|| "<no value>".to_string());
                let prop_owner = lambda_obj.property_values.get(idx)
                    .map(|pv| pv.owner)
                    .unwrap_or(lambda_obj.owner);
                let prop_perms = lambda_obj.property_values.get(idx)
                    .map(|pv| pv.perms)
                    .unwrap_or(0);

                MooProperty {
                    name: prop.name.clone(),
                    value: prop_value,
                    owner: prop_owner,
                    permissions: prop_perms,
                }
            }).collect();

            let parsed_data = ParsedObjectData {
                name: lambda_obj.name.clone(),
                flags: lambda_obj.flags,
                owner: lambda_obj.owner,
                location: lambda_obj.location,
                parent: lambda_obj.parent,
                verbs,
                properties,
                defined_prop_count: lambda_obj.properties.len(),
            };

            objects.insert(*obj_id, MooObject {
                id: *obj_id,
                state: ObjectState::Parsed(parsed_data),
            });
        }

        // Check if object #1 exists
        if objects.contains_key(&1) {
        // info!("✓ Object #1 was found in proper parser");
        } else {
            // error!("✗ Object #1 was NOT found in proper parser!");
        }

        all_object_ids.sort();

        let total_time = parse_start.elapsed();
        // info!("Total database parsing completed in {:?} for {} (proper parser)", total_time, name);
        // info!("✓ Final check: Object #1 is in the database with name: {}",
        //      objects.get(&1).map(|obj| obj.name()).unwrap_or("NOT FOUND".to_string()));

        Ok(MooDatabase {
            name: lambda_db.name,
            path: lambda_db.path,
            version: lambda_db.version.as_numeric(),
            total_objects: lambda_db.total_objects,
            total_verbs: lambda_db.total_verbs,
            total_players: lambda_db.total_players,
            players: lambda_db.players,
            objects,
            verb_code_map: lambda_db.verb_programs,
        })
    }

    pub fn parse_database_full(&self, path: &str, name: &str) -> Result<MooDatabase> {
        let parse_start = Instant::now();
        // info!("Starting to parse database: {}", name);

        // Start with basic parsing for immediate display
        let file_read_start = Instant::now();
        let content = fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();
        // info!("Read file {} in {:?}, {} lines", path, file_read_start.elapsed(), lines.len());

        if lines.is_empty() {
            return Err(anyhow!("Empty database file"));
        }

        // Parse header
        let header = lines[0];
        let version = if let Some(pos) = header.find("Format Version ") {
            let version_str = &header[pos + 15..];
            version_str.split_whitespace()
                .next()
                .and_then(|s| s.trim_end_matches("**").parse::<i32>().ok())
                .unwrap_or(1)
        } else {
            return Err(anyhow!("Unknown format: {}", header));
        };

        // Parse counts
        let total_objects: i64 = lines.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let total_verbs: i64 = lines.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        let _dummy: i64 = lines.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
        let total_players: i64 = lines.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);

        // Parse player list
        let mut players = Vec::new();
        let mut line_idx = 5;
        for _ in 0..total_players {
            if let Some(line) = lines.get(line_idx) {
                if let Ok(player_id) = line.parse::<i64>() {
                    players.push(player_id);
                }
                line_idx += 1;
            }
        }

        // First pass: Parse verb code section to get all verb code
        let verb_parse_start = Instant::now();
        // info!("Starting verb code parsing phase");
        let mut verb_code_map: HashMap<(i64, String), String> = HashMap::new();
        let mut object_end_idx = line_idx;

        // Scan through to find where objects end and verb code begins
        let mut temp_idx = line_idx;
        let mut failed_objects = Vec::new();

        while temp_idx < lines.len() {
            let line = lines[temp_idx].trim();

            // Keep track of where objects end
            if line.len() > 1 && line.starts_with('#') && line[1..].chars().all(|c| c.is_ascii_digit()) {
                object_end_idx = temp_idx;
            }

            // Look for verb code section
            if let Ok(_verb_count) = lines[temp_idx].parse::<i64>() {
                if temp_idx + 1 < lines.len() && lines[temp_idx + 1].starts_with('#') && lines[temp_idx + 1].contains(':') {
                    // Found the verb code section
                    temp_idx += 1; // Skip the count

                    // Parse all verb programs
                    let mut verb_count_parsed = 0;
                    while temp_idx < lines.len() {
                        if let Some((obj_id, _verb_name, code, new_idx)) = self.parse_verb_code(&lines, temp_idx)? {
                            // Store with verb index
                            let verb_count = verb_code_map.iter().filter(|((id, _), _)| *id == obj_id).count();
                            verb_code_map.insert((obj_id, verb_count.to_string()), code);
                            temp_idx = new_idx;
                            verb_count_parsed += 1;
                            if verb_count_parsed % 100 == 0 {
        // debug!("Parsed {} verb codes so far...", verb_count_parsed);
                            }
                        } else {
                            break;
                        }
                    }
                    break;
                }
            }
            temp_idx += 1;
        }

        // info!("Verb code parsing completed in {:?}, found {} verb codes", verb_parse_start.elapsed(), verb_code_map.len());
        // info!("Object section ends at line {} (object_end_idx)", object_end_idx + 1);

        // Second pass: Create placeholders for all objects, then parse on demand
        let object_parse_start = Instant::now();
        // info!("Starting object parsing phase");
        let mut objects = HashMap::new();
        let mut all_object_ids = Vec::new();

        // First, collect all object IDs and create placeholders
        let placeholder_start = Instant::now();
        let mut scan_idx = line_idx;
        while scan_idx < lines.len() {
            let line = lines[scan_idx].trim();

            // Object definitions are lines that contain ONLY #<number>
            if line.len() > 1 && line.starts_with('#') && line[1..].chars().all(|c| c.is_ascii_digit()) {
                if let Ok(obj_id) = line[1..].parse::<i64>() {
                    all_object_ids.push(obj_id);
                    // Create placeholder
                    objects.insert(obj_id, MooObject::new_placeholder(obj_id));
                }
            }

            // Stop when we hit verb code section
            if let Ok(_verb_count) = line.parse::<i64>() {
                if scan_idx + 1 < lines.len() && lines[scan_idx + 1].starts_with('#') && lines[scan_idx + 1].contains(':') {
                    break;
                }
            }
            scan_idx += 1;
        }

        // info!("Created {} object placeholders in {:?}", all_object_ids.len(), placeholder_start.elapsed());
        // debug!("Object IDs: {:?}",
        //         if all_object_ids.len() <= 20 { format!("{:?}", all_object_ids) }
        //         else { format!("{:?}...{:?}", &all_object_ids[..10], &all_object_ids[all_object_ids.len()-10..]) });

        // Check if object #1 was found
        if objects.contains_key(&1) {
        // info!("✓ Object #1 was found in placeholder phase");
        } else {
            // error!("✗ Object #1 was NOT found in placeholder phase!");
        }

        // Now parse each object individually
        line_idx = 5 + total_players as usize; // Reset to start of objects
        let mut parsed_count = 0;
        let individual_parse_start = Instant::now();
        let mut parse_times = Vec::new();

        while line_idx < lines.len() && parsed_count < total_objects {
            let line = lines[line_idx].trim();

            // Object definitions are lines that contain ONLY #<number>
            if line.len() > 1 && line.starts_with('#') && line[1..].chars().all(|c| c.is_ascii_digit()) {
                if let Ok(obj_id) = line[1..].parse::<i64>() {
                    let obj_parse_start = Instant::now();
                    match self.parse_single_object(&lines, line_idx, version, &verb_code_map) {
                        Ok(Some((parsed_data, new_idx))) => {
                            // Update the placeholder with parsed data
                            if let Some(obj) = objects.get_mut(&obj_id) {
                                obj.state = ObjectState::Parsed(parsed_data);
                            }
                            line_idx = new_idx;
                            parsed_count += 1;
                            let obj_parse_time = obj_parse_start.elapsed();
                            parse_times.push(obj_parse_time);
                            if parsed_count % 10 == 0 {
        // debug!("Parsed {} objects so far...", parsed_count);
                            }
                            continue;
                        }
                        Ok(None) => {
        // debug!("parse_single_object returned None for object #{} at line {}", obj_id, line_idx);
                            if let Some(obj) = objects.get_mut(&obj_id) {
                                obj.state = ObjectState::ParseFailed("Parser returned None".to_string());
                            }
                            line_idx += 1;
                            continue;
                        }
                        Err(e) => {
                            // Mark object as failed to parse
                            if let Some(obj) = objects.get_mut(&obj_id) {
                                obj.state = ObjectState::ParseFailed(e.to_string());
                            }
                            failed_objects.push((obj_id, e.to_string()));
                            // error!("Failed to parse object #{}: {}", obj_id, e);
                            line_idx += 1;
                            continue;
                        }
                    }
                }
            }
            line_idx += 1;

            // Skip over verb code sections if we encounter them
            if line_idx + 1 < lines.len() {
                let line = lines[line_idx].trim();
                let next_line = lines[line_idx + 1].trim();

                // Check for verb code section: number followed by #obj:verb
                if let Ok(verb_count) = line.parse::<i64>() {
                    if next_line.starts_with('#') && next_line.contains(':') {
        // info!("Found verb code section at line {} - skipping {} verb codes", line_idx + 1, verb_count);
                        // Skip the verb count line
                        line_idx += 1;

                        // Skip all the verb code entries
                        let mut verbs_skipped = 0;
                        while line_idx < lines.len() && verbs_skipped < verb_count {
                            let current_line = lines[line_idx].trim();
                            if current_line.starts_with('#') && current_line.contains(':') {
                                // Skip to the end of this verb code
                                line_idx += 1;
                                // Skip the verb code content until we find the next verb or end
                                while line_idx < lines.len() {
                                    let code_line = lines[line_idx].trim();
                                    line_idx += 1;
                                    if code_line == "." {
                                        // End of verb code
                                        verbs_skipped += 1;
                                        break;
                                    }
                                }
                            } else {
                                line_idx += 1;
                            }
                        }
        // info!("Skipped {} verb codes, continuing object parsing from line {}", verbs_skipped, line_idx + 1);
                        continue; // Continue parsing objects after the verb section
                    }
                }
            }
        }

        let avg_parse_time = if !parse_times.is_empty() {
            let total: std::time::Duration = parse_times.iter().sum();
            total / parse_times.len() as u32
        } else {
            std::time::Duration::from_secs(0)
        };

        // info!("Object parsing completed in {:?}, parsed {} objects", individual_parse_start.elapsed(), parsed_count);
        // info!("Average time per object: {:?}", avg_parse_time);

        // Post-process to resolve inherited property names for parsed objects
        let inheritance_start = Instant::now();
        // info!("Starting property inheritance resolution");
        self.resolve_inherited_property_names_lazy(&mut objects);
        // info!("Property inheritance resolution completed in {:?}", inheritance_start.elapsed());

        // Log parsing statistics for debugging
        if !failed_objects.is_empty() {
            // error!("Failed to parse {} objects: {:?}", failed_objects.len(), failed_objects);
        }

        if objects.len() < total_objects as usize {
            // warn!("Expected {} objects, but only parsed {}. Missing: {}",
            //          total_objects, objects.len(), total_objects as usize - objects.len());
        }

        let total_parse_time = parse_start.elapsed();
        // info!("Total database parsing completed in {:?} for {}", total_parse_time, name);

        // Final check for object #1
        if objects.contains_key(&1) {
            if let Some(obj1) = objects.get(&1) {
        // info!("✓ Final check: Object #1 is in the database with name: {}", obj1.name());
            }
        } else {
            // error!("✗ Final check: Object #1 is NOT in the final database!");
        }

        // Log first few object IDs in the final database
        let mut final_ids: Vec<i64> = objects.keys().copied().collect();
        final_ids.sort();
        // info!("First 10 objects in final database: {:?}", &final_ids[..10.min(final_ids.len())]);

        Ok(MooDatabase {
            name: name.to_string(),
            path: path.to_string(),
            version,
            total_objects,
            total_verbs,
            total_players,
            players,
            objects,
            verb_code_map,
        })
    }

    fn parse_single_object(&self, lines: &[&str], start_idx: usize, _version: i32, verb_code_map: &HashMap<(i64, String), String>) -> Result<Option<(ParsedObjectData, usize)>> {
        if start_idx >= lines.len() {
            return Ok(None);
        }

        // Parse object ID
        let obj_line = lines[start_idx];
        let obj_id = match obj_line.strip_prefix('#').and_then(|s| s.parse::<i64>().ok()) {
            Some(id) => id,
            None => return Ok(None), // Not a valid object definition, skip it
        };

        let mut idx = start_idx + 1; // Skip the object ID line

        // Check for recycled
        if idx < lines.len() && lines[idx].trim() == "recycled" {
            return Ok(Some((ParsedObjectData {
                name: "Recycled Object".to_string(),
                flags: 0,
                owner: -1,
                location: -1,
                parent: -1,
                verbs: Vec::new(),
                properties: Vec::new(),
                defined_prop_count: 0,
            }, idx + 1)));
        }

        // Parse object name
        let name = if idx < lines.len() {
            lines[idx].to_string()
        } else {
            "Unknown Object".to_string()
        };
        idx += 1;

        // Skip old handles line (usually empty)
        if idx < lines.len() {
            idx += 1;
        }

        // Parse object fields
        let flags = if idx < lines.len() {
            lines[idx].parse::<i64>().unwrap_or(0)
        } else { 0 };
        idx += 1;

        let owner = if idx < lines.len() {
            lines[idx].parse::<i64>().unwrap_or(-1)
        } else { -1 };
        idx += 1;

        let location = if idx < lines.len() {
            lines[idx].parse::<i64>().unwrap_or(-1)
        } else { -1 };
        idx += 1;

        // Skip contents and next fields
        idx += 2;

        let parent = if idx < lines.len() {
            lines[idx].parse::<i64>().unwrap_or(-1)
        } else { -1 };
        idx += 1;

        // Skip child and sibling fields for now
        idx += 2;

        // Parse verb count
        let verb_count = if idx < lines.len() {
            lines[idx].parse::<usize>().unwrap_or(0)
        } else { 0 };
        idx += 1;

        // Parse verbs
        let mut verbs = Vec::new();
        for verb_idx in 0..verb_count {
            if idx + 3 < lines.len() {
                let verb_name = lines[idx].to_string();
                let verb_owner = lines[idx + 1].parse::<i64>().unwrap_or(-1);
                let verb_perms = lines[idx + 2].parse::<i64>().unwrap_or(0);
                let _verb_prep = lines[idx + 3].parse::<i64>().unwrap_or(0);

                // Look up the actual verb code using object ID and verb index
                let code = verb_code_map.get(&(obj_id, verb_idx.to_string()))
                    .cloned()
                    .unwrap_or_else(|| "// Verb code not found".to_string());

                verbs.push(MooVerb {
                    name: verb_name,
                    owner: verb_owner,
                    permissions: verb_perms,
                    code,
                });
                idx += 4;
            } else {
        // debug!("Incomplete verb data for verb {} at line {}", verb_idx, idx);
                break; // Stop parsing verbs but continue with the object
            }
        }

        // Parse property definitions
        let prop_def_count = if idx < lines.len() {
            lines[idx].parse::<usize>().unwrap_or(0)
        } else { 0 };
        idx += 1;

        let mut prop_names = Vec::new();
        for _ in 0..prop_def_count {
            if idx < lines.len() {
                prop_names.push(lines[idx].to_string());
                idx += 1;
            }
        }

        // Parse property values
        let prop_val_count = if idx < lines.len() {
            lines[idx].parse::<usize>().unwrap_or(0)
        } else { 0 };
        idx += 1;

        let mut properties = Vec::new();
        for i in 0..prop_val_count {
            if idx + 2 < lines.len() {
                let (prop_name, is_inherited) = if i < prop_names.len() {
                    (prop_names[i].clone(), false)
                } else {
                    (format!("property_{}", i), true)
                };
                let prop_value = self.parse_property_value(&lines, &mut idx);
                let prop_owner = if idx < lines.len() {
                    lines[idx].parse::<i64>().unwrap_or(-1)
                } else { -1 };
                idx += 1;
                let prop_perms = if idx < lines.len() {
                    lines[idx].parse::<i64>().unwrap_or(0)
                } else { 0 };
                idx += 1;

                properties.push(MooProperty {
                    name: if is_inherited {
                        format!("{} (inherited)", prop_name)
                    } else {
                        prop_name
                    },
                    value: prop_value,
                    owner: prop_owner,
                    permissions: prop_perms,
                });
            } else {
        // debug!("Insufficient data for property {} at line {}", i, idx);
                break;
            }
        }

        Ok(Some((ParsedObjectData {
            name,
            flags,
            owner,
            location,
            parent,
            verbs,
            properties,
            defined_prop_count: prop_def_count,
        }, idx)))
    }

    fn parse_property_value(&self, lines: &[&str], idx: &mut usize) -> String {
        if *idx >= lines.len() {
            return "null".to_string();
        }

        // Parse MOO value type
        let value_type = lines[*idx].parse::<i64>().unwrap_or(-1);
        *idx += 1;

        match value_type {
            -2 => "CLEAR".to_string(), // TYPE_CLEAR
            -1 => "null".to_string(),  // TYPE_NONE
            0 => {                     // TYPE_INT
                if *idx < lines.len() {
                    let val = lines[*idx].to_string();
                    *idx += 1;
                    val
                } else {
                    "0".to_string()
                }
            }
            1 => {                     // TYPE_OBJ
                if *idx < lines.len() {
                    let val = format!("#{}", lines[*idx]);
                    *idx += 1;
                    val
                } else {
                    "#-1".to_string()
                }
            }
            2 => {                     // TYPE_STR
                if *idx < lines.len() {
                    let val = format!("\"{}\"", lines[*idx]);
                    *idx += 1;
                    val
                } else {
                    "\"\"".to_string()
                }
            }
            3 => {                     // TYPE_ERR
                if *idx < lines.len() {
                    let err_num = lines[*idx].parse::<i32>().unwrap_or(0);
                    *idx += 1;

                    // Map error numbers to MOO error names
                    let err_name = match err_num {
                        0 => "E_NONE",
                        1 => "E_TYPE",
                        2 => "E_DIV",
                        3 => "E_PERM",
                        4 => "E_PROPNF",
                        5 => "E_VERBNF",
                        6 => "E_VARNF",
                        7 => "E_INVIND",
                        8 => "E_RECMOVE",
                        9 => "E_MAXREC",
                        10 => "E_RANGE",
                        11 => "E_ARGS",
                        12 => "E_NACC",
                        13 => "E_INVARG",
                        14 => "E_QUOTA",
                        15 => "E_FLOAT",
                        _ => return format!("E_UNKNOWN_{}", err_num),
                    };
                    err_name.to_string()
                } else {
                    "E_NONE".to_string()
                }
            }
            9 => {                     // TYPE_FLOAT
                if *idx < lines.len() {
                    let val = lines[*idx].to_string();
                    *idx += 1;
                    val
                } else {
                    "0.0".to_string()
                }
            }
            4 => {                     // TYPE_LIST
                let list_len = if *idx < lines.len() {
                    lines[*idx].parse::<usize>().unwrap_or(0)
                } else { 0 };
                *idx += 1;

                let mut items = Vec::new();
                let mut all_strings = true;

                for _ in 0..list_len {
                    let item = self.parse_property_value(lines, idx);
                    // Check if this item is a string (starts and ends with quotes)
                    if !item.starts_with('"') || !item.ends_with('"') {
                        all_strings = false;
                    }
                    items.push(item);
                }

                // If all items are strings, format as concatenated strings with newlines
                if all_strings && list_len > 0 {
                    let string_contents: Vec<String> = items.into_iter()
                        .map(|s| {
                            // Remove quotes and unescape
                            if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
                                s[1..s.len()-1].to_string()
                            } else {
                                s
                            }
                        })
                        .collect();
                    format!("LIST OF STRINGS:\n{}", string_contents.join("\n"))
                } else {
                    format!("{{{}}}", items.join(", "))
                }
            }
            5 => {                     // TYPE_MAP
                // Maps are stored as a list of key-value pairs
                let map_len = if *idx < lines.len() {
                    lines[*idx].parse::<usize>().unwrap_or(0)
                } else { 0 };
                *idx += 1;

                let mut pairs = Vec::new();
                for _ in 0..map_len {
                    // Parse key
                    let key = self.parse_property_value(lines, idx);
                    // Parse value
                    let value = self.parse_property_value(lines, idx);
                    pairs.push(format!("{}: {}", key, value));
                }
                format!("MAP{{{}}}", pairs.join(", "))
            }
            _ => format!("unknown_type_{}", value_type),
        }
    }


    fn resolve_inherited_property_names_lazy(&self, objects: &mut HashMap<i64, MooObject>) {
        // First pass: Extract defined property names for each parsed object
        let mut object_defined_props: HashMap<i64, Vec<String>> = HashMap::new();
        for (id, obj) in objects.iter() {
            if let Some(data) = obj.parsed_data() {
                let mut defined_props = Vec::new();
                for i in 0..data.defined_prop_count {
                    if let Some(prop) = data.properties.get(i) {
                        defined_props.push(prop.name.clone());
                    }
                }
                object_defined_props.insert(*id, defined_props);
            }
        }

        // Second pass: Build complete property name lists by walking inheritance
        let mut complete_prop_names: HashMap<i64, Vec<String>> = HashMap::new();

        fn build_property_list(
            obj_id: i64,
            objects: &HashMap<i64, MooObject>,
            defined_props: &HashMap<i64, Vec<String>>,
            cache: &mut HashMap<i64, Vec<String>>
        ) -> Vec<String> {
            if let Some(cached) = cache.get(&obj_id) {
                return cached.clone();
            }

            let mut all_props = Vec::new();

            if let Some(obj) = objects.get(&obj_id) {
                if let Some(data) = obj.parsed_data() {
                    // Start with parent's properties if there is one
                    if data.parent > 0 && data.parent != obj_id && objects.contains_key(&data.parent) {
                        all_props = build_property_list(data.parent, objects, defined_props, cache);
                    }

                    // Add properties defined on this object
                    if let Some(defined) = defined_props.get(&obj_id) {
                        all_props.extend(defined.iter().cloned());
                    }
                }
            }

            cache.insert(obj_id, all_props.clone());
            all_props
        }

        // Build complete property lists for all parsed objects
        let obj_ids: Vec<i64> = objects.keys().cloned().collect();
        for obj_id in obj_ids {
            build_property_list(obj_id, objects, &object_defined_props, &mut complete_prop_names);
        }

        // Third pass: Update property names in parsed objects
        for (id, obj) in objects.iter_mut() {
            if let ObjectState::Parsed(data) = &mut obj.state {
                if let Some(all_names) = complete_prop_names.get(id) {
                    for (i, prop) in data.properties.iter_mut().enumerate() {
                        if let Some(real_name) = all_names.get(i) {
                            // Determine if this property is inherited
                            let is_inherited = i >= data.defined_prop_count;

                            prop.name = if is_inherited {
                                format!("{} (inherited)", real_name)
                            } else {
                                real_name.clone()
                            };
                        }
                    }
                }
            }
        }
    }

    fn parse_verb_code(&self, lines: &[&str], start_idx: usize) -> Result<Option<(i64, String, String, usize)>> {
        if start_idx >= lines.len() {
            return Ok(None);
        }

        let header = lines[start_idx];
        if let Some(colon_pos) = header.find(':') {
            let obj_part = &header[1..colon_pos]; // Skip '#'
            let verb_name = &header[colon_pos + 1..];

            if let Ok(obj_id) = obj_part.parse::<i64>() {
                let mut idx = start_idx + 1;
                let mut code_lines = Vec::new();

                // Read until we find a line with just "."
                while idx < lines.len() {
                    if lines[idx] == "." {
                        break;
                    }
                    code_lines.push(lines[idx]);
                    idx += 1;
                }

                let code = if code_lines.is_empty() {
                    "// Empty verb".to_string()
                } else {
                    code_lines.join("\n")
                };

                return Ok(Some((obj_id, verb_name.to_string(), code, idx + 1)));
            }
        }

        Ok(None)
    }
}

/// Browser application state
#[allow(dead_code)]
struct App {
    databases: Vec<MooDatabase>,
    current_db: usize,
    selected_object: Option<i64>,
    object_list_state: ListState,
    detail_scroll: u16,
    detail_scroll_state: ScrollbarState,
    current_view: ViewMode,
    current_tab: DetailTab,
    show_db_selector: bool,
    db_selector_state: ListState,
    quit: bool,
    focused_panel: FocusedPanel,  // Which panel has focus in the three-panel view
    parse_progress: Option<String>,  // Current parsing status
    middle_pane_selection: MiddlePaneSelection,  // Which verb/property is selected
    middle_pane_scroll: u16,
    middle_pane_state: ListState,
}

#[derive(Debug, Clone, PartialEq)]
enum MiddlePaneSelection {
    None,
    Verb(usize),
    Property(usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FocusedPanel {
    ObjectList,    // Left panel
    MiddlePane,    // Properties/Verbs panel
    DetailView,    // Right panel with code/details
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ViewMode {
    ObjectList,
    ObjectDetail,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum DetailTab {
    Overview,
    Properties,
    Verbs,
    Relationships,
}

impl App {
    fn new() -> Self {
        let mut app = Self {
            databases: Vec::new(),
            current_db: 0,
            selected_object: None,
            object_list_state: ListState::default(),
            detail_scroll: 0,
            detail_scroll_state: ScrollbarState::default(),
            current_view: ViewMode::ObjectList,
            current_tab: DetailTab::Overview,
            show_db_selector: false,
            db_selector_state: ListState::default(),
            quit: false,
            focused_panel: FocusedPanel::ObjectList,
            parse_progress: None,
            middle_pane_selection: MiddlePaneSelection::None,
            middle_pane_scroll: 0,
            middle_pane_state: ListState::default(),
        };
        app.object_list_state.select(Some(0));
        app.db_selector_state.select(Some(0));
        app
    }

    fn load_databases(&mut self) -> Result<()> {
        let total_start = Instant::now();
        let parser = MooDatabaseParser::new();
        let db_files = vec![
            ("examples/Minimal.db", "Minimal MOO"),
            ("examples/LambdaCore-latest.db", "LambdaCore"),
            ("examples/toastcore.db", "ToastCore"),
            ("examples/JHCore-DEV-2.db", "JaysHouseCore"),
        ];

        // info!("Loading databases from files: {:?}", db_files);

        for (path, name) in db_files {
            let db_start = Instant::now();
        // info!("Checking path: {}", path);
            if Path::new(path).exists() {
        // info!("Found {}, parsing...", path);
                match parser.parse_database_proper(path, name) {
                    Ok(db) => {
                        let db_duration = db_start.elapsed();
        // info!("Successfully loaded {} with {} objects in {:?}", name, db.objects.len(), db_duration);
                        self.databases.push(db);
                    },
                    Err(_e) => {} // error!("Failed to load {}: {}", name, e),
                }
            } else {
                // warn!("Database file {} not found", path);
            }
        }

        if self.databases.is_empty() {
            return Err(anyhow!("No databases found"));
        }

        let total_duration = total_start.elapsed();
        // info!("Total database loading time: {:?}", total_duration);

        Ok(())
    }

    fn current_database(&self) -> Option<&MooDatabase> {
        self.databases.get(self.current_db)
    }

    fn switch_database(&mut self, new_db: usize) {
        if new_db < self.databases.len() {
            self.current_db = new_db;
            self.selected_object = None;
            self.object_list_state.select(Some(0));
            self.detail_scroll = 0;
            self.current_tab = DetailTab::Overview;
            self.update_selected_object();
        }
    }

    fn next_object(&mut self) {
        if let Some(db) = self.current_database() {
            let object_count = db.objects.len();
            if object_count > 0 {
                let current = self.object_list_state.selected().unwrap_or(0);
                let next = if current >= object_count - 1 { 0 } else { current + 1 };
                self.object_list_state.select(Some(next));
                self.update_selected_object();
            }
        }
    }

    fn previous_object(&mut self) {
        if let Some(db) = self.current_database() {
            let object_count = db.objects.len();
            if object_count > 0 {
                let current = self.object_list_state.selected().unwrap_or(0);
                let prev = if current == 0 { object_count - 1 } else { current - 1 };
                self.object_list_state.select(Some(prev));
                self.update_selected_object();
            }
        }
    }

    fn update_selected_object(&mut self) {
        if let Some(db) = self.current_database() {
            if let Some(selected) = self.object_list_state.selected() {
                let mut object_ids: Vec<i64> = db.objects.keys().copied().collect();
                object_ids.sort();
                if let Some(&obj_id) = object_ids.get(selected) {
                    self.selected_object = Some(obj_id);
                    self.detail_scroll = 0;
                    // Reset middle pane selection when switching objects
                    self.middle_pane_selection = MiddlePaneSelection::None;
                    self.middle_pane_scroll = 0;
                }
            }
        }
    }

    fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            DetailTab::Overview => DetailTab::Properties,
            DetailTab::Properties => DetailTab::Verbs,
            DetailTab::Verbs => DetailTab::Relationships,
            DetailTab::Relationships => DetailTab::Overview,
        };
        self.detail_scroll = 0;
    }

    fn previous_tab(&mut self) {
        self.current_tab = match self.current_tab {
            DetailTab::Overview => DetailTab::Relationships,
            DetailTab::Relationships => DetailTab::Verbs,
            DetailTab::Verbs => DetailTab::Properties,
            DetailTab::Properties => DetailTab::Overview,
        };
        self.detail_scroll = 0;
    }

    fn scroll_detail_up(&mut self) {
        if self.detail_scroll > 0 {
            self.detail_scroll -= 1;
        }
    }

    fn scroll_detail_down(&mut self) {
        self.detail_scroll += 1;
    }

    fn page_down_detail(&mut self) {
        // Scroll down by approximately one page (20 lines)
        self.detail_scroll = self.detail_scroll.saturating_add(20);
    }

    fn page_up_detail(&mut self) {
        // Scroll up by approximately one page (20 lines)
        self.detail_scroll = self.detail_scroll.saturating_sub(20);
    }

    fn navigate_middle_pane_up(&mut self) {
        if let Some(db) = self.current_database() {
            if let Some(obj_id) = self.selected_object {
                if let Some(obj) = db.objects.get(&obj_id) {
                    if let Some(data) = obj.parsed_data() {
                        // Calculate current position
                        let total_props = data.properties.len();
                        let total_verbs = data.verbs.len();

                    match &self.middle_pane_selection {
                        MiddlePaneSelection::None => {
                            // Start from the bottom
                            if total_verbs > 0 {
                                self.middle_pane_selection = MiddlePaneSelection::Verb(total_verbs - 1);
                            } else if total_props > 0 {
                                self.middle_pane_selection = MiddlePaneSelection::Property(total_props - 1);
                            }
                        }
                        MiddlePaneSelection::Property(idx) => {
                            if *idx > 0 {
                                self.middle_pane_selection = MiddlePaneSelection::Property(idx - 1);
                            }
                        }
                        MiddlePaneSelection::Verb(idx) => {
                            if *idx > 0 {
                                self.middle_pane_selection = MiddlePaneSelection::Verb(idx - 1);
                            } else if total_props > 0 {
                                self.middle_pane_selection = MiddlePaneSelection::Property(total_props - 1);
                            }
                        }
                    }
                    }
                }
            }
        }
    }

    fn navigate_middle_pane_down(&mut self) {
        if let Some(db) = self.current_database() {
            if let Some(obj_id) = self.selected_object {
                if let Some(obj) = db.objects.get(&obj_id) {
                    if let Some(data) = obj.parsed_data() {
                        let total_props = data.properties.len();
                        let total_verbs = data.verbs.len();

                    match &self.middle_pane_selection {
                        MiddlePaneSelection::None => {
                            // Start from the top
                            if total_props > 0 {
                                self.middle_pane_selection = MiddlePaneSelection::Property(0);
                            } else if total_verbs > 0 {
                                self.middle_pane_selection = MiddlePaneSelection::Verb(0);
                            }
                        }
                        MiddlePaneSelection::Property(idx) => {
                            if idx + 1 < total_props {
                                self.middle_pane_selection = MiddlePaneSelection::Property(idx + 1);
                            } else if total_verbs > 0 {
                                self.middle_pane_selection = MiddlePaneSelection::Verb(0);
                            }
                        }
                        MiddlePaneSelection::Verb(idx) => {
                            if idx + 1 < total_verbs {
                                self.middle_pane_selection = MiddlePaneSelection::Verb(idx + 1);
                            }
                        }
                    }
                    }
                }
            }
        }
    }

    fn page_down_middle_pane(&mut self) {
        // Page down by 10 items
        for _ in 0..10 {
            self.navigate_middle_pane_down();
        }
    }

    fn next_db_in_selector(&mut self) {
        let db_count = self.databases.len();
        if db_count > 0 {
            let current = self.db_selector_state.selected().unwrap_or(0);
            let next = if current >= db_count - 1 { 0 } else { current + 1 };
            self.db_selector_state.select(Some(next));
        }
    }

    fn previous_db_in_selector(&mut self) {
        let db_count = self.databases.len();
        if db_count > 0 {
            let current = self.db_selector_state.selected().unwrap_or(0);
            let prev = if current == 0 { db_count - 1 } else { current - 1 };
            self.db_selector_state.select(Some(prev));
        }
    }

    fn select_current_db(&mut self) {
        if let Some(selected) = self.db_selector_state.selected() {
            self.switch_database(selected);
            self.show_db_selector = false;
        }
    }
}

fn main() -> Result<()> {
    // Check for debug flag
    let args: Vec<String> = std::env::args().collect();
    let debug_list = args.contains(&"--debug-list".to_string());

    let app_start = Instant::now();

    // Initialize tracing to write logs to file
    // use tracing_subscriber::fmt::writer::MakeWriterExt;
    // let log_file = std::fs::OpenOptions::new()
    //     .create(true)
    //     .write(true)
    //     .truncate(true)
    //     .open("moo_parser_debug.log")?;
    //
    // tracing_subscriber::fmt()
    //     .with_writer(log_file.with_max_level(tracing::Level::DEBUG))
    //     .with_ansi(false)
    //     .init();

        // info!("Starting MOO Database Browser");

    // Create app first (before terminal setup)
    let app_create_start = Instant::now();
    let mut app = App::new();
        // info!("App creation took {:?}", app_create_start.elapsed());

    let db_load_start = Instant::now();
    app.load_databases()?;
        // info!("Database loading took {:?}", db_load_start.elapsed());

    // Debug output if requested
    if debug_list {
        // Debug output removed - app continues to UI
    }

    // Setup terminal (only if not in debug mode)
    let terminal_setup_start = Instant::now();
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
        // info!("Terminal setup completed in {:?}", terminal_setup_start.elapsed());

    app.update_selected_object();

        // info!("Total startup time before UI loop: {:?}", app_start.elapsed());

    // Run app
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    let mut first_draw = true;
    let ui_start = Instant::now();

    loop {
        let draw_start = Instant::now();
        terminal.draw(|f| ui(f, app))?;

        if first_draw {
            let draw_time = draw_start.elapsed();
        // info!("First UI draw completed in {:?}", draw_time);
        // info!("Time from UI loop start to first draw: {:?}", ui_start.elapsed());
            first_draw = false;
        }

        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                    if app.show_db_selector {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                app.show_db_selector = false;
                            }
                            KeyCode::Up => app.previous_db_in_selector(),
                            KeyCode::Down => app.next_db_in_selector(),
                            KeyCode::Enter => app.select_current_db(),
                            _ => {}
                        }
                    } else {
                        match app.current_view {
                            ViewMode::Help => match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => {
                                    app.current_view = ViewMode::ObjectList;
                                }
                                _ => {}
                            },
                            ViewMode::ObjectDetail => match key.code {
                                KeyCode::Char('q') => app.quit = true,
                                KeyCode::Esc => app.current_view = ViewMode::ObjectList,
                                KeyCode::Up => {
                                    match app.focused_panel {
                                        FocusedPanel::ObjectList => app.previous_object(),
                                        FocusedPanel::MiddlePane => app.navigate_middle_pane_up(),
                                        FocusedPanel::DetailView => app.scroll_detail_up(),
                                    }
                                },
                                KeyCode::Down => {
                                    match app.focused_panel {
                                        FocusedPanel::ObjectList => app.next_object(),
                                        FocusedPanel::MiddlePane => app.navigate_middle_pane_down(),
                                        FocusedPanel::DetailView => app.scroll_detail_down(),
                                    }
                                },
                                KeyCode::Left => {
                                    app.focused_panel = match app.focused_panel {
                                        FocusedPanel::DetailView => FocusedPanel::MiddlePane,
                                        FocusedPanel::MiddlePane => FocusedPanel::ObjectList,
                                        FocusedPanel::ObjectList => FocusedPanel::ObjectList,
                                    };
                                },
                                KeyCode::Right => {
                                    app.focused_panel = match app.focused_panel {
                                        FocusedPanel::ObjectList => FocusedPanel::MiddlePane,
                                        FocusedPanel::MiddlePane => FocusedPanel::DetailView,
                                        FocusedPanel::DetailView => FocusedPanel::DetailView,
                                    };
                                },
                                KeyCode::Tab => app.next_tab(),
                                KeyCode::BackTab => app.previous_tab(),
                                KeyCode::Char('d') => app.show_db_selector = true,
                                KeyCode::Char('h') | KeyCode::F(1) => app.current_view = ViewMode::Help,
                                KeyCode::Char(' ') => {
                                    // Page down
                                    match app.focused_panel {
                                        FocusedPanel::DetailView => app.page_down_detail(),
                                        FocusedPanel::MiddlePane => app.page_down_middle_pane(),
                                        _ => {}
                                    }
                                },
                                _ => {}
                            },
                            ViewMode::ObjectList => match key.code {
                                KeyCode::Char('q') => app.quit = true,
                                KeyCode::Up => app.previous_object(),
                                KeyCode::Down => app.next_object(),
                                KeyCode::Enter => app.current_view = ViewMode::ObjectDetail,
                                KeyCode::Char('d') => app.show_db_selector = true,
                                KeyCode::Char('h') | KeyCode::F(1) => app.current_view = ViewMode::Help,
                                _ => {}
                            },
                        }
                    }
                    }
                }
                Event::Mouse(mouse) => {
                    match mouse.kind {
                        MouseEventKind::Down(_) => {
                            // Handle mouse click - determine which panel was clicked
                            let mouse_x = mouse.column;
                            let mouse_y = mouse.row;

                            if app.current_view == ViewMode::ObjectDetail {
                                // Three-panel layout: Object list (0-20%), Middle pane (20-50%), Detail view (50-100%)
                                // We'll use approximate boundaries since we can't access f.area() here
                                let _terminal_width = 100u16; // Assume roughly 100 columns for now
                                let left_boundary = 20;
                                let middle_boundary = 50;

                                if mouse_x < left_boundary {
                                    // Clicked in object list
                                    app.focused_panel = FocusedPanel::ObjectList;
                                    // Calculate which object was clicked
                                    if mouse_y > 3 { // Account for header
                                        let clicked_idx = (mouse_y - 4) as usize; // Adjust for header and border
                                        if let Some(db) = app.current_database() {
                                            let mut object_ids: Vec<i64> = db.objects.keys().copied().collect();
                                            object_ids.sort();
                                            if clicked_idx < object_ids.len() {
                                                app.object_list_state.select(Some(clicked_idx));
                                                app.update_selected_object();
                                            }
                                        }
                                    }
                                } else if mouse_x < middle_boundary {
                                    // Clicked in middle pane
                                    app.focused_panel = FocusedPanel::MiddlePane;
                                    // Calculate which property/verb was clicked
                                    if mouse_y > 3 {
                                        let clicked_idx = (mouse_y - 4) as usize;
                                        if let Some(obj_id) = app.selected_object {
                                            if let Some(db) = app.current_database() {
                                                if let Some(obj) = db.objects.get(&obj_id) {
                                                    if let Some(data) = obj.parsed_data() {
                                                        let total_props = data.properties.len();
                                                        // Account for headers and spacing (rough calculation)
                                                        if clicked_idx > 0 && clicked_idx <= total_props {
                                                            app.middle_pane_selection = MiddlePaneSelection::Property(clicked_idx - 1);
                                                        } else if clicked_idx > total_props + 2 && clicked_idx <= total_props + 2 + data.verbs.len() {
                                                            app.middle_pane_selection = MiddlePaneSelection::Verb(clicked_idx - total_props - 3);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    // Clicked in detail view
                                    app.focused_panel = FocusedPanel::DetailView;
                                }
                            } else if app.current_view == ViewMode::ObjectList {
                                // Single panel view - handle object selection
                                if mouse_y > 3 {
                                    let clicked_idx = (mouse_y - 4) as usize;
                                    if let Some(db) = app.current_database() {
                                        let mut object_ids: Vec<i64> = db.objects.keys().copied().collect();
                                        object_ids.sort();
                                        if clicked_idx < object_ids.len() {
                                            app.object_list_state.select(Some(clicked_idx));
                                            app.update_selected_object();
                                        }
                                    }
                                }
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            match app.focused_panel {
                                FocusedPanel::ObjectList => app.next_object(),
                                FocusedPanel::MiddlePane => app.navigate_middle_pane_down(),
                                FocusedPanel::DetailView => app.scroll_detail_down(),
                            }
                        }
                        MouseEventKind::ScrollUp => {
                            match app.focused_panel {
                                FocusedPanel::ObjectList => app.previous_object(),
                                FocusedPanel::MiddlePane => app.navigate_middle_pane_up(),
                                FocusedPanel::DetailView => app.scroll_detail_up(),
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if app.quit {
            break;
        }
    }
    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    match app.current_view {
        ViewMode::Help => render_help(f, app),
        ViewMode::ObjectList => render_object_list(f, app),
        ViewMode::ObjectDetail => render_object_detail(f, app),
    }

    // Render database selector modal if active
    if app.show_db_selector {
        render_db_selector(f, app);
    }
}

fn render_middle_pane(f: &mut Frame, app: &mut App, area: Rect, obj: &MooObject) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Header for middle pane
    let header = Paragraph::new(" Properties & Verbs ")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(header, chunks[0]);

    // Content area split into two
    let _content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Combined list of properties and verbs
    let mut items = Vec::new();
    let mut current_selection_idx = 0;

    // Add properties header
    items.push(ListItem::new("── Properties ──").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));

    // Add properties
    if let Some(data) = obj.parsed_data() {
        for (i, prop) in data.properties.iter().enumerate() {
        let is_selected = matches!(app.middle_pane_selection, MiddlePaneSelection::Property(idx) if idx == i);
        let style = if is_selected {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        items.push(ListItem::new(format!("  {} {}",
            if is_selected { "▶" } else { " " },
            prop.name
        )).style(style));

        if matches!(app.middle_pane_selection, MiddlePaneSelection::Property(idx) if idx == i) {
            current_selection_idx = items.len() - 1;
        }
    }

    // Add spacing
    items.push(ListItem::new(""));

    // Add verbs header
    items.push(ListItem::new("── Verbs ──").style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));

    // Add verbs
    for (i, verb) in data.verbs.iter().enumerate() {
        let is_selected = matches!(app.middle_pane_selection, MiddlePaneSelection::Verb(idx) if idx == i);
        let style = if is_selected {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        items.push(ListItem::new(format!("  {} {}",
            if is_selected { "▶" } else { " " },
            verb.name
        )).style(style));

        if matches!(app.middle_pane_selection, MiddlePaneSelection::Verb(idx) if idx == i) {
            current_selection_idx = items.len() - 1;
        }
    }
    }

    // Determine border style based on focus
    let border_style = if app.focused_panel == FocusedPanel::MiddlePane {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let list = List::new(items)
        .block(Block::default()
            .title(" Properties & Verbs ")
            .borders(Borders::ALL)
            .border_style(border_style))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    // Update the list state to reflect the current selection
    app.middle_pane_state.select(Some(current_selection_idx));

    f.render_stateful_widget(list, chunks[1], &mut app.middle_pane_state);
}

fn render_help(f: &mut Frame, _app: &mut App) {
    let area = f.area();
    let block = Block::default()
        .title(" Enhanced MOO Database Browser - Help ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled("Enhanced MOO Database Browser Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  ↑/↓      - Navigate objects"),
        Line::from("  Enter    - View object details"),
        Line::from("  d        - Database selector"),
        Line::from("  Esc      - Go back"),
        Line::from(""),
        Line::from("Object Detail View (Three-Panel Layout):"),
        Line::from("  ←/→      - Switch focus between panels"),
        Line::from("  ↑/↓      - Navigate in focused panel"),
        Line::from("  Tab      - Next tab (Overview/Properties/Verbs/Relations)"),
        Line::from("  Shift-Tab - Previous tab"),
        Line::from("  d        - Database selector"),
        Line::from("  Esc      - Back to object list"),
        Line::from(""),
        Line::from("Database Selector:"),
        Line::from("  ↑/↓      - Navigate databases"),
        Line::from("  Enter    - Select database"),
        Line::from("  Esc      - Close selector"),
        Line::from(""),
        Line::from("Detail Tabs:"),
        Line::from("  Overview     - Basic object information"),
        Line::from("  Properties   - Property names and values"),
        Line::from("  Verbs        - Verb names and code"),
        Line::from("  Relationships - Parent/child hierarchy"),
        Line::from(""),
        Line::from("General:"),
        Line::from("  h/F1     - Show this help"),
        Line::from("  q        - Quit"),
        Line::from(""),
        Line::from("Press Esc or q to close this help screen"),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: true });

    let popup_area = centered_rect(70, 80, area);
    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}

fn render_object_list(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(f.area());

    // Header
    if let Some(db) = app.current_database() {
        let header = Paragraph::new(format!(
            " {} (v{}) - {}/{} objects, {} verbs, {} players | Press 'd' for database selector, 'h' for help ",
            db.name, db.version, db.objects.len(), db.total_objects, db.total_verbs, db.total_players
        ))
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));

        f.render_widget(header, chunks[0]);
    }

    // Object list
    if let Some(db) = app.current_database() {
        // Debug terminal/chunk info
        // debug!("Object list chunk height: {} lines", chunks[1].height);

        let mut items: Vec<ListItem> = Vec::new();
        let mut object_ids: Vec<i64> = db.objects.keys().copied().collect();
        object_ids.sort();

        // Debug log first few object IDs
        // debug!("Building UI object list with {} objects. First 10 IDs: {:?}",
        //        object_ids.len(), &object_ids[..10.min(object_ids.len())]);

        // Log what we're about to render
        if object_ids.len() > 0 && object_ids[0] == 0 {
        // debug!("First 5 objects being added to UI list:");
            for (i, &id) in object_ids.iter().take(5).enumerate() {
        // debug!("  Position {}: Object #{}", i, id);
            }
        }

        for obj_id in object_ids {
            if let Some(obj) = db.objects.get(&obj_id) {
                let is_player = db.players.contains(&obj_id);
                let player_marker = if is_player { " [P]" } else { "" };

                let (prop_count, verb_count, parent_info) = match &obj.state {
                    ObjectState::Parsed(data) => {
                        let parent_info = if data.parent >= 0 {
                            format!(" (parent: #{})", data.parent)
                        } else {
                            String::new()
                        };
                        (data.properties.len(), data.verbs.len(), parent_info)
                    },
                    _ => (0, 0, String::new()),
                };

                let style = if is_player {
                    Style::default().fg(Color::Green)
                } else if obj_id < 10 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };

                items.push(ListItem::new(format!(
                    "#{:3} {}{}{} [{} props, {} verbs]",
                    obj_id, obj.name(), player_marker, parent_info, prop_count, verb_count
                )).style(style));
            }
        }

        // Debug: log what's actually in the items list
        if items.len() > 0 {
        // debug!("Final UI items list has {} items. First few items:", items.len());
            for (i, item) in items.iter().take(3).enumerate() {
        // debug!("  Item {}: (ListItem content not directly accessible)", i);
            }
        }

        // Determine border style based on focus
        let border_style = if app.focused_panel == FocusedPanel::ObjectList {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let objects_list = List::new(items)
            .block(Block::default()
                .title(" Objects (Enter to view details) ")
                .borders(Borders::ALL)
                .border_style(border_style))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("► ");

        f.render_stateful_widget(objects_list, chunks[1], &mut app.object_list_state);
    }
}

fn render_object_detail(f: &mut Frame, app: &mut App) {
    // Three-panel layout: Objects | Properties/Verbs | Details/Code
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),  // Object list
            Constraint::Percentage(30),  // Properties/Verbs list
            Constraint::Percentage(50),  // Details/Code view
        ])
        .split(f.area());

    // Object list (left pane)
    render_object_list_pane(f, app, chunks[0]);

    // Middle pane: Properties/Verbs list
    if let Some(obj_id) = app.selected_object {
        if let Some(db) = app.current_database() {
            if let Some(obj) = db.objects.get(&obj_id) {
                let obj_clone = obj.clone();
                // db goes out of scope here
                render_middle_pane(f, app, chunks[1], &obj_clone);
            }
        }
    }

    // Right pane: Details/Code view
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(chunks[2]);

    // Tab bar for right pane
    let tab_titles = ["Overview", "Property Details", "Verb Code", "Relationships"];

    // Determine border style based on focus
    let tab_border_style = if app.focused_panel == FocusedPanel::DetailView {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).border_style(tab_border_style))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .select(app.current_tab as usize);

    f.render_widget(tabs, right_chunks[0]);

    // Tab content
    let current_tab = app.current_tab;
    let detail_scroll = app.detail_scroll;

    if let Some(obj_id) = app.selected_object {
        if let Some(db) = app.current_database() {
            if let Some(obj) = db.objects.get(&obj_id) {
                // Clone the necessary data to avoid borrow checker issues
                let obj_clone = obj.clone();
                let db_objects = db.objects.clone();
                let db_verb_code_map = db.verb_code_map.clone();
                let db_players = db.players.clone();

                // Show details based on middle pane selection
                match &app.middle_pane_selection {
                    MiddlePaneSelection::Property(idx) => {
                        render_selected_property(f, right_chunks[1], &obj_clone, *idx, detail_scroll, &mut app.detail_scroll_state);
                    }
                    MiddlePaneSelection::Verb(idx) => {
                        render_selected_verb(f, right_chunks[1], &obj_clone, *idx, &db_verb_code_map, detail_scroll, &mut app.detail_scroll_state);
                    }
                    MiddlePaneSelection::None => {
                        // Show current tab when nothing is selected
                        match current_tab {
                            DetailTab::Overview => render_overview_tab(f, right_chunks[1], &obj_clone, &db_objects, &db_players, detail_scroll, &mut app.detail_scroll_state),
                            DetailTab::Properties => render_properties_tab(f, right_chunks[1], &obj_clone, detail_scroll, &mut app.detail_scroll_state),
                            DetailTab::Verbs => render_verbs_tab(f, right_chunks[1], &obj_clone, &db_verb_code_map, detail_scroll, &mut app.detail_scroll_state),
                            DetailTab::Relationships => render_relationships_tab(f, right_chunks[1], &obj_clone, &db_objects, detail_scroll, &mut app.detail_scroll_state),
                        }
                    }
                }
            }
        }
    }
}

fn render_overview_tab(f: &mut Frame, area: Rect, obj: &MooObject, db_objects: &HashMap<i64, MooObject>, db_players: &Vec<i64>, detail_scroll: u16, detail_scroll_state: &mut ScrollbarState) {
    let mut details = Vec::new();

    details.push(Line::from(Span::styled(
        format!("Object #{}: {}", obj.id, obj.name()),
        Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)
    )));
    details.push(Line::from(""));

    match &obj.state {
        ObjectState::Placeholder => {
            details.push(Line::from("Loading object data..."));
        }
        ObjectState::ParseFailed(error) => {
            details.push(Line::from(Span::styled(
                format!("Parse failed: {}", error),
                Style::default().fg(Color::Red)
            )));
        }
        ObjectState::Parsed(data) => {
            details.push(Line::from(Span::styled("Basic Information:", Style::default().add_modifier(Modifier::UNDERLINED))));
            details.push(Line::from(format!("ID: #{}", obj.id)));
            details.push(Line::from(format!("Name: {}", data.name)));
            details.push(Line::from(format!("Flags: {}", format_object_flags(data.flags))));
            details.push(Line::from(format!("Owner: #{}", data.owner)));
            details.push(Line::from(format!("Location: #{}", data.location)));

            if data.parent >= 0 {
                let parent_name = db_objects.get(&data.parent)
                    .map(|p| p.name())
                    .unwrap_or("Unknown".to_string());
                details.push(Line::from(format!("Parent: #{} ({})", data.parent, parent_name)));
            } else {
                details.push(Line::from("Parent: None"));
            }

            if db_players.contains(&obj.id) {
                details.push(Line::from(Span::styled("Type: Player", Style::default().fg(Color::Green))));
            } else {
                details.push(Line::from("Type: Object"));
            }

            details.push(Line::from(""));
            details.push(Line::from(format!("Properties: {}", data.properties.len())));
            details.push(Line::from(format!("Verbs: {}", data.verbs.len())));
            details.push(Line::from(""));
            details.push(Line::from(Span::styled("Description:", Style::default().add_modifier(Modifier::UNDERLINED))));
            details.push(Line::from(format!("Object #{} with {} verbs and {} properties", obj.id, data.verbs.len(), data.properties.len())));
        }
    }

    let details_len = details.len();

    let paragraph = Paragraph::new(details)
        .block(Block::default()
            .title(" Overview (↑/↓ to scroll, Tab for next tab) ")
            .borders(Borders::ALL))
        .wrap(Wrap { trim: true })
        .scroll((detail_scroll, 0));

    f.render_widget(paragraph, area);

    // Scrollbar
    render_scrollbar(f, area, details_len, detail_scroll, detail_scroll_state);
}

fn render_properties_tab(f: &mut Frame, area: Rect, obj: &MooObject, detail_scroll: u16, detail_scroll_state: &mut ScrollbarState) {
    let mut details = Vec::new();

    details.push(Line::from(Span::styled(
        format!("Properties for #{}: {}", obj.id, obj.name()),
        Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)
    )));
    details.push(Line::from(""));

    match &obj.state {
        ObjectState::Placeholder => {
            details.push(Line::from("Loading properties..."));
        }
        ObjectState::ParseFailed(error) => {
            details.push(Line::from(Span::styled(
                format!("Parse failed: {}", error),
                Style::default().fg(Color::Red)
            )));
        }
        ObjectState::Parsed(data) => {
            if data.properties.is_empty() {
                details.push(Line::from("No properties defined."));
            } else {
                // Dense format: show properties with two-line style for large values
                for prop in data.properties.iter() {
                    let name_color = if prop.name.contains("(inherited)") {
                        Color::Gray
                    } else {
                        Color::Yellow
                    };

                    // Use two-line format for values longer than 50 characters
                    if prop.value.len() > 50 {
                        // Line 1: name, owner, permissions
                        details.push(Line::from(vec![
                            Span::styled(&prop.name, Style::default().fg(name_color)),
                            Span::raw(format!(" (#{}) [{}]", prop.owner, format_property_permissions_compact(prop.permissions)))
                        ]));

                        // Line 2: indented value
                        details.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(&prop.value, Style::default().fg(Color::Cyan))
                        ]));
                        details.push(Line::from(""));
                    } else {
                        // Single line format for short values
                        details.push(Line::from(vec![
                            Span::styled(
                                format!("{:<30}", prop.name),
                                Style::default().fg(name_color)
                            ),
                            Span::raw(format!(" {:<25} #{:<3} {}",
                                prop.value,
                                prop.owner,
                                format_property_permissions_compact(prop.permissions)
                            ))
                        ]));
                    }
                }

                // Add section for property details if there are inherited ones
                let inherited_count = data.properties.iter()
                    .filter(|p| p.name.contains("(inherited)"))
                    .count();
                let defined_count = data.defined_prop_count;

                details.push(Line::from(""));
                details.push(Line::from(format!(
                    "Properties: {} defined on this object, {} inherited ({} total)",
                    defined_count,
                    inherited_count,
                    data.properties.len()
                )));
            }
        }
    }

    let details_len = details.len();

    let paragraph = Paragraph::new(details)
        .block(Block::default()
            .title(" Properties (↑/↓ to scroll, Tab for next tab) ")
            .borders(Borders::ALL))
        .wrap(Wrap { trim: true })
        .scroll((detail_scroll, 0));

    f.render_widget(paragraph, area);
    render_scrollbar(f, area, details_len, detail_scroll, detail_scroll_state);
}

fn render_verbs_tab(f: &mut Frame, area: Rect, obj: &MooObject, db_verb_code_map: &HashMap<(i64, String), String>, detail_scroll: u16, detail_scroll_state: &mut ScrollbarState) {
    let mut details = Vec::new();

    details.push(Line::from(Span::styled(
        format!("Verbs for #{}: {}", obj.id, obj.name()),
        Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)
    )));
    details.push(Line::from(""));

    match &obj.state {
        ObjectState::Placeholder => {
            details.push(Line::from("Loading verbs..."));
        }
        ObjectState::ParseFailed(error) => {
            details.push(Line::from(Span::styled(
                format!("Parse failed: {}", error),
                Style::default().fg(Color::Red)
            )));
        }
        ObjectState::Parsed(data) => {
            if data.verbs.is_empty() {
                details.push(Line::from("No verbs defined."));
            } else {
                for (i, verb) in data.verbs.iter().enumerate() {
                    details.push(Line::from(Span::styled(
                        format!("Verb #{}: {}", i + 1, verb.name),
                        Style::default().add_modifier(Modifier::BOLD).fg(Color::Green)
                    )));
                    details.push(Line::from(format!("  Owner: #{}", verb.owner)));
                    details.push(Line::from(format!("  Permissions: {}", format_verb_permissions_compact(verb.permissions))));

                    // Use the code that was already looked up during parsing
                    let code = verb.code.clone();

                    // Pretty-print the code
                    let pretty_code = pretty_print_moo_code(&code);

                    details.push(Line::from("  Code:"));
                    for line in pretty_code.lines() {
                        details.push(Line::from(format!("    {}", line)));
                    }
                    details.push(Line::from(""));
                }
            }
        }
    }

    let details_len = details.len();

    let paragraph = Paragraph::new(details)
        .block(Block::default()
            .title(" Verbs (↑/↓ to scroll, Tab for next tab) ")
            .borders(Borders::ALL))
        .wrap(Wrap { trim: true })
        .scroll((detail_scroll, 0));

    f.render_widget(paragraph, area);
    render_scrollbar(f, area, details_len, detail_scroll, detail_scroll_state);
}

fn render_relationships_tab(f: &mut Frame, area: Rect, obj: &MooObject, db_objects: &HashMap<i64, MooObject>, detail_scroll: u16, detail_scroll_state: &mut ScrollbarState) {
    let mut details = Vec::new();

    details.push(Line::from(Span::styled(
        format!("Relationships for #{}: {}", obj.id, obj.name()),
        Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)
    )));
    details.push(Line::from(""));

    match &obj.state {
        ObjectState::Placeholder => {
            details.push(Line::from("Loading relationships..."));
        }
        ObjectState::ParseFailed(error) => {
            details.push(Line::from(Span::styled(
                format!("Parse failed: {}", error),
                Style::default().fg(Color::Red)
            )));
        }
        ObjectState::Parsed(data) => {
            // Parent
            if data.parent >= 0 {
                if let Some(parent_obj) = db_objects.get(&data.parent) {
                    details.push(Line::from(Span::styled("Parent:", Style::default().add_modifier(Modifier::UNDERLINED))));
                    details.push(Line::from(format!("  #{} - {}", data.parent, parent_obj.name())));
                }
            } else {
                details.push(Line::from(Span::styled("Parent:", Style::default().add_modifier(Modifier::UNDERLINED))));
                details.push(Line::from("  None (root object)"));
            }
            details.push(Line::from(""));

            // Children
            let mut children: Vec<i64> = db_objects.values()
                .filter_map(|o| {
                    if let Some(o_data) = o.parsed_data() {
                        if o_data.parent == obj.id {
                            Some(o.id)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            children.sort();

            if children.is_empty() {
                details.push(Line::from(Span::styled("Children:", Style::default().add_modifier(Modifier::UNDERLINED))));
                details.push(Line::from("  None"));
            } else {
                details.push(Line::from(Span::styled(
                    format!("Children ({}):", children.len()),
                    Style::default().add_modifier(Modifier::UNDERLINED)
                )));
                for child_id in children {
                    if let Some(child_obj) = db_objects.get(&child_id) {
                        details.push(Line::from(format!("  #{} - {}", child_id, child_obj.name())));
                    }
                }
            }
            details.push(Line::from(""));

            // Location
            if data.location >= 0 {
                if let Some(location_obj) = db_objects.get(&data.location) {
                    details.push(Line::from(Span::styled("Location:", Style::default().add_modifier(Modifier::UNDERLINED))));
                    details.push(Line::from(format!("  #{} - {}", data.location, location_obj.name())));
                    details.push(Line::from(""));
                }
            }
        }
    }

    let details_len = details.len();

    let paragraph = Paragraph::new(details)
        .block(Block::default()
            .title(" Relationships (↑/↓ to scroll, Tab for next tab) ")
            .borders(Borders::ALL))
        .wrap(Wrap { trim: true })
        .scroll((detail_scroll, 0));

    f.render_widget(paragraph, area);
    render_scrollbar(f, area, details_len, detail_scroll, detail_scroll_state);
}

fn render_selected_property(f: &mut Frame, area: Rect, obj: &MooObject, prop_idx: usize, detail_scroll: u16, detail_scroll_state: &mut ScrollbarState) {
    let mut details = Vec::new();

    match &obj.state {
        ObjectState::Placeholder => {
            details.push(Line::from("Loading property data..."));
        }
        ObjectState::ParseFailed(error) => {
            details.push(Line::from(Span::styled(
                format!("Parse failed: {}", error),
                Style::default().fg(Color::Red)
            )));
        }
        ObjectState::Parsed(data) => {
            if let Some(prop) = data.properties.get(prop_idx) {
                details.push(Line::from(Span::styled(
                    format!("Property: {}", prop.name),
                    Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow)
                )));
                details.push(Line::from(""));

                details.push(Line::from(format!("Value: {}", prop.value)));
                details.push(Line::from(format!("Owner: #{}", prop.owner)));
                details.push(Line::from(format!("Permissions: {}", format_property_permissions_compact(prop.permissions))));

                if prop.name.contains("(inherited)") {
                    details.push(Line::from(""));
                    details.push(Line::from(Span::styled(
                        "This property is inherited from a parent object",
                        Style::default().fg(Color::Gray)
                    )));
                }
            } else {
                details.push(Line::from("Property not found"));
            }
        }
    }

    let details_len = details.len();

    let paragraph = Paragraph::new(details)
        .block(Block::default()
            .title(" Property Details ")
            .borders(Borders::ALL))
        .wrap(Wrap { trim: true })
        .scroll((detail_scroll, 0));

    f.render_widget(paragraph, area);
    render_scrollbar(f, area, details_len, detail_scroll, detail_scroll_state);
}

fn render_selected_verb(f: &mut Frame, area: Rect, obj: &MooObject, verb_idx: usize, verb_code_map: &HashMap<(i64, String), String>, detail_scroll: u16, detail_scroll_state: &mut ScrollbarState) {
    let mut details = Vec::new();

    match &obj.state {
        ObjectState::Placeholder => {
            details.push(Line::from("Loading verb data..."));
        }
        ObjectState::ParseFailed(error) => {
            details.push(Line::from(Span::styled(
                format!("Parse failed: {}", error),
                Style::default().fg(Color::Red)
            )));
        }
        ObjectState::Parsed(data) => {
            if let Some(verb) = data.verbs.get(verb_idx) {
                details.push(Line::from(Span::styled(
                    format!("Verb: {}", verb.name),
                    Style::default().add_modifier(Modifier::BOLD).fg(Color::Green)
                )));
                details.push(Line::from(""));

                details.push(Line::from(format!("Owner: #{}", verb.owner)));
                details.push(Line::from(format!("Permissions: {}", format_verb_permissions_compact(verb.permissions))));
                details.push(Line::from(""));

                // Get actual code from the verb_code_map using verb index
                let code = verb_code_map.get(&(obj.id, verb_idx.to_string()))
                    .cloned()
                    .unwrap_or_else(|| verb.code.clone());

                // Pretty-print the code
                let pretty_code = pretty_print_moo_code(&code);

                details.push(Line::from(Span::styled("Code:", Style::default().add_modifier(Modifier::BOLD))));
                for line in pretty_code.lines() {
                    details.push(Line::from(line.to_string()));
                }
            } else {
                details.push(Line::from("Verb not found"));
            }
        }
    }

    let details_len = details.len();

    let paragraph = Paragraph::new(details)
        .block(Block::default()
            .title(" Verb Details ")
            .borders(Borders::ALL))
        .wrap(Wrap { trim: true })
        .scroll((detail_scroll, 0));

    f.render_widget(paragraph, area);
    render_scrollbar(f, area, details_len, detail_scroll, detail_scroll_state);
}

fn render_scrollbar(f: &mut Frame, area: Rect, content_length: usize, detail_scroll: u16, detail_scroll_state: &mut ScrollbarState) {
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    let scrollbar_area = area.inner(Margin {
        horizontal: 0,
        vertical: 1,
    });

    *detail_scroll_state = detail_scroll_state
        .content_length(content_length.saturating_sub(1))
        .position(detail_scroll as usize);

    f.render_stateful_widget(scrollbar, scrollbar_area, detail_scroll_state);
}

fn render_object_list_pane(f: &mut Frame, app: &mut App, area: Rect) {
    if let Some(db) = app.current_database() {
        let mut items: Vec<ListItem> = Vec::new();
        let mut object_ids: Vec<i64> = db.objects.keys().copied().collect();
        object_ids.sort();

        for obj_id in object_ids {
            if let Some(obj) = db.objects.get(&obj_id) {
                let is_player = db.players.contains(&obj_id);
                let player_marker = if is_player { " [P]" } else { "" };

                let style = if is_player {
                    Style::default().fg(Color::Green)
                } else if obj_id < 10 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };

                let selected_style = if Some(obj_id) == app.selected_object {
                    style.add_modifier(Modifier::REVERSED)
                } else {
                    style
                };

                let display_name = obj.name();
                items.push(ListItem::new(format!(
                    "#{:3} {}{}",
                    obj_id,
                    if display_name.len() > 20 {
                        format!("{}...", &display_name[..17])
                    } else {
                        display_name
                    },
                    player_marker
                )).style(selected_style));
            }
        }

        // Determine border style based on focus
        let border_style = if app.focused_panel == FocusedPanel::ObjectList {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let objects_list = List::new(items)
            .block(Block::default()
                .title(format!(" {} Objects ", db.name))
                .borders(Borders::ALL)
                .border_style(border_style))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("► ");

        f.render_stateful_widget(objects_list, area, &mut app.object_list_state);
    }
}

fn render_db_selector(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let popup_area = centered_rect(50, 60, area);

    f.render_widget(Clear, popup_area);

    let mut items: Vec<ListItem> = Vec::new();
    for (i, db) in app.databases.iter().enumerate() {
        let marker = if i == app.current_db { " (current)" } else { "" };
        let item_text = format!("{} - {}/{} objects{}", db.name, db.objects.len(), db.total_objects, marker);
        let style = if i == app.current_db {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        items.push(ListItem::new(item_text).style(style));
    }

    let db_list = List::new(items)
        .block(Block::default()
            .title(" Select Database (Enter to switch, Esc to cancel) ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Yellow)))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("► ");

    f.render_stateful_widget(db_list, popup_area, &mut app.db_selector_state);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

// Tests module
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_pretty_print_moo_code() {
        let code = "if (player && connection)\nfor i in [1..10]\nj = i * 2;\nendfor\nendif";
        let result = pretty_print_moo_code(code);
        let expected = vec![
            "if (player && connection)",
            "  for i in [1..10]",
            "    j = i * 2;",
            "  endfor",
            "endif"
        ].join("\n");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_moo_database_parser_basic() {
        let parser = MooDatabaseParser::new();

        // Test basic property value parsing logic
        let lines = vec!["2", "test string"];
        let mut idx = 0;
        let result = parser.parse_property_value(&lines, &mut idx);
        assert_eq!(result, "\"test string\"");
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_moo_object_creation() {
        let obj = MooObject {
            id: 1,
            state: ObjectState::Parsed(ParsedObjectData {
                name: "Test Object".to_string(),
                flags: 0,
                owner: 0,
                location: -1,
                parent: 0,
                verbs: vec![],
                properties: vec![],
                defined_prop_count: 0,
            }),
        };

        assert_eq!(obj.id, 1);
        assert_eq!(obj.name(), "Test Object");
        if let Some(data) = obj.parsed_data() {
            assert_eq!(data.properties.len(), 0);
        }
    }

    #[test]
    fn test_app_navigation() {
        let mut app = App::new();

        // Test that app starts with object list selected
        assert_eq!(app.object_list_state.selected(), Some(0));
        assert_eq!(app.current_view, ViewMode::ObjectList);
        assert_eq!(app.focused_panel, FocusedPanel::ObjectList);
    }

    #[test]
    fn test_moo_database_counts() {
        let db = MooDatabase {
            name: "Test DB".to_string(),
            path: "test.db".to_string(),
            version: 1,
            total_objects: 100,
            total_verbs: 50,
            total_players: 5,
            players: vec![1, 2, 3, 4, 5],
            objects: HashMap::new(),
            verb_code_map: HashMap::new(),
        };

        assert_eq!(db.total_objects, 100);
        assert_eq!(db.total_players, 5);
        assert_eq!(db.players.len(), 5);
    }

    #[test]
    fn test_list_string_formatting() {
        let parser = MooDatabaseParser::new();

        // Test list of strings formatting
        let lines = vec!["4", "3", "2", "first", "2", "second", "2", "third"];
        let mut idx = 0;
        let result = parser.parse_property_value(&lines, &mut idx);

        assert!(result.starts_with("LIST OF STRINGS:"));
        assert!(result.contains("first"));
        assert!(result.contains("second"));
        assert!(result.contains("third"));
    }

    #[test]
    fn test_focused_panel_switching() {
        let mut app = App::new();

        // Test panel focus navigation
        app.focused_panel = FocusedPanel::ObjectList;

        // Simulate right arrow key press logic
        app.focused_panel = match app.focused_panel {
            FocusedPanel::ObjectList => FocusedPanel::MiddlePane,
            FocusedPanel::MiddlePane => FocusedPanel::DetailView,
            FocusedPanel::DetailView => FocusedPanel::DetailView,
        };

        assert_eq!(app.focused_panel, FocusedPanel::MiddlePane);
    }

    #[test]
    fn test_error_code_mapping() {
        let parser = MooDatabaseParser::new();

        let lines = vec!["3", "4"]; // TYPE_ERR, E_PROPNF
        let mut idx = 0;
        let result = parser.parse_property_value(&lines, &mut idx);
        assert_eq!(result, "E_PROPNF");
    }

    #[test]
    fn test_object_placeholder_creation() {
        let obj = MooObject::new_placeholder(42);
        assert_eq!(obj.id, 42);
        assert!(matches!(obj.state, ObjectState::Placeholder));
        assert_eq!(obj.name(), "Object #42 (loading...)");
        assert!(obj.parsed_data().is_none());
    }

    #[test]
    fn test_object_parse_failed_state() {
        let mut obj = MooObject::new_placeholder(5);
        obj.state = ObjectState::ParseFailed("Test error".to_string());
        assert_eq!(obj.name(), "Object #5 (parse failed: Test error)");
        assert!(obj.parsed_data().is_none());
    }

    #[test]
    fn test_object_id_line_detection() {
        // Test the pattern used to detect object definition lines
        let test_cases = vec![
            ("#0", true),
            ("#1", true),
            ("#123", true),
            ("#999999", true),
            ("# 1", false),      // Space after #
            (" #1", false),      // Leading space
            ("#1 ", false),      // Trailing space
            ("#1abc", false),    // Letters after number
            ("#", false),        // No number
            ("##1", false),      // Double hash
            ("#-1", false),      // Negative (actually invalid in MOO)
            ("#1.5", false),     // Decimal
            ("123", false),      // No hash
            ("", false),         // Empty
        ];

        for (line, expected) in test_cases {
            let result = line.len() > 1 &&
                        line.starts_with('#') &&
                        line[1..].chars().all(|c| c.is_ascii_digit());
            assert_eq!(result, expected,
                      "Object ID detection failed for '{}': expected {}, got {}",
                      line, expected, result);
        }
    }

    #[test]
    fn test_verb_code_section_detection() {
        // Test the pattern for detecting the start of verb code section
        let test_cases = vec![
            (vec!["1727", "#0:0"], true),              // Valid: number + verb reference
            (vec!["456", "#5:initialize"], true),      // Valid: number + verb with name
            (vec!["0", "#1:test"], true),              // Valid: zero verbs
            (vec!["abc", "#0:0"], false),              // Invalid: not a number
            (vec!["123", "not a verb"], false),        // Invalid: not a verb reference
            (vec!["123", "#0"], false),                // Invalid: no colon
            (vec!["123"], false),                      // Invalid: only one line
        ];

        for (lines, expected) in test_cases {
            let result = if lines.len() >= 2 {
                lines[0].parse::<i64>().is_ok() &&
                lines[1].starts_with('#') &&
                lines[1].contains(':')
            } else {
                false
            };
            assert_eq!(result, expected,
                      "Verb section detection failed for {:?}", lines);
        }
    }

    #[test]
    fn test_moo_object_list_sorting() {
        let mut objects = HashMap::new();
        objects.insert(5, MooObject::new_placeholder(5));
        objects.insert(1, MooObject::new_placeholder(1));
        objects.insert(3, MooObject::new_placeholder(3));
        objects.insert(0, MooObject::new_placeholder(0));
        objects.insert(2, MooObject::new_placeholder(2));

        let mut object_ids: Vec<i64> = objects.keys().copied().collect();
        object_ids.sort();

        assert_eq!(object_ids, vec![0, 1, 2, 3, 5]);
    }

    #[test]
    fn test_app_state_initialization() {
        let app = App::new();

        // Check initial state
        assert_eq!(app.current_db, 0);
        assert_eq!(app.selected_object, None);
        assert_eq!(app.object_list_state.selected(), Some(0));
        assert_eq!(app.detail_scroll, 0);
        assert_eq!(app.current_view, ViewMode::ObjectList);
        assert_eq!(app.current_tab, DetailTab::Overview);
        assert!(!app.show_db_selector);
        assert!(!app.quit);
        assert_eq!(app.focused_panel, FocusedPanel::ObjectList);
        assert_eq!(app.middle_pane_selection, MiddlePaneSelection::None);
    }

    #[test]
    fn test_pretty_print_nested_structures() {
        let code = "if (a)\nif (b)\nc = 1;\nelse\nc = 2;\nendif\nendif";
        let result = pretty_print_moo_code(code);
        let expected = vec![
            "if (a)",
            "  if (b)",
            "    c = 1;",
            "    else",
            "    c = 2;",
            "  endif",
            "endif"
        ].join("\n");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pretty_print_try_except() {
        let code = "try\ndo_something();\nexcept e (E_PERM)\nhandle_error(e);\nfinally\ncleanup();\nendtry";
        let result = pretty_print_moo_code(code);
        let expected = vec![
            "try",
            "  do_something();",
            "  except e (E_PERM)",
            "  handle_error(e);",
            "  finally",
            "  cleanup();",
            "endtry"
        ].join("\n");
        assert_eq!(result, expected);
    }
}
