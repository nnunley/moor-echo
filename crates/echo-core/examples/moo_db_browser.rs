use std::collections::HashMap;
use std::fs;
use std::io::stdout;
use std::path::Path;
use std::time::Duration;

use anyhow::{anyhow, Result};

/// Pretty-print MOO code with proper indentation
fn pretty_print_moo_code(code: &str) -> String {
    let mut result = Vec::new();
    let mut indent_level: usize = 0;
    let indent = "  "; // 2 spaces per indent level
    
    for line in code.lines() {
        let trimmed = line.trim();
        
        // Decrease indent for closing keywords
        if trimmed == "endif" || trimmed == "endfor" || trimmed == "endwhile" 
            || trimmed == "endtry" || trimmed == "endfork" {
            indent_level = indent_level.saturating_sub(1);
        }
        
        // Add the line with current indentation
        result.push(format!("{}{}", indent.repeat(indent_level), trimmed));
        
        // Increase indent after opening keywords
        if trimmed.starts_with("if ") || trimmed == "else" || trimmed.starts_with("elseif ")
            || trimmed.starts_with("for ") || trimmed.starts_with("while ")
            || trimmed.starts_with("try") || trimmed.starts_with("except ")
            || trimmed.starts_with("finally") || trimmed.starts_with("fork ") {
            indent_level += 1;
        }
        
        // Handle else/elseif/except/finally (they close the previous block and open a new one)
        if trimmed == "else" || trimmed.starts_with("elseif ") 
            || trimmed.starts_with("except ") || trimmed == "finally" {
            // We already increased, so decrease by 1 to stay at same level
            indent_level = indent_level.saturating_sub(1);
        }
    }
    
    result.join("\n")
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
struct MooVerb {
    name: String,
    owner: i64,
    permissions: String,
    code: String,
}

/// Represents a MOO property with actual value
#[derive(Debug, Clone)]
struct MooProperty {
    name: String,
    value: String,
    owner: i64,
    permissions: String,
}

/// Represents a MOO object with full details
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct MooObject {
    id: i64,
    name: String,
    flags: i64,
    owner: i64,
    location: i64,
    parent: i64,
    children: Vec<i64>,
    verbs: Vec<MooVerb>,
    properties: Vec<MooProperty>,
    defined_prop_count: usize,  // Number of properties defined on this object
    description: String,
}

/// Represents a MOO database with full parsing
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct MooDatabase {
    name: String,
    path: String,
    version: i32,
    total_objects: i64,
    total_verbs: i64,
    total_players: i64,
    players: Vec<i64>,
    objects: HashMap<i64, MooObject>,
    verb_code_map: HashMap<(i64, String), String>,
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
        let db_files = vec![
            ("examples/Minimal.db", "Minimal MOO"),
            ("examples/LambdaCore-latest.db", "LambdaCore"),
            ("examples/toastcore.db", "ToastCore"),
            ("examples/JHCore-DEV-2.db", "JaysHouseCore"),
        ];

        for (path, name) in db_files {
            if Path::new(path).exists() {
                match self.parse_database_full(path, name) {
                    Ok(db) => self.databases.push(db),
                    Err(e) => eprintln!("Failed to load {}: {}", name, e),
                }
            }
        }

        if self.databases.is_empty() {
            return Err(anyhow!("No databases found"));
        }

        Ok(())
    }

    fn parse_database_full(&self, path: &str, name: &str) -> Result<MooDatabase> {
        // Start with basic parsing for immediate display
        let content = fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();
        
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
        let mut verb_code_map: HashMap<(i64, String), String> = HashMap::new();
        let mut object_end_idx = line_idx;
        
        // Scan through to find where objects end and verb code begins
        let mut temp_idx = line_idx;
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
                    while temp_idx < lines.len() {
                        if let Some((obj_id, _verb_name, code, new_idx)) = self.parse_verb_code(&lines, temp_idx)? {
                            // Store with verb index
                            let verb_count = verb_code_map.iter().filter(|((id, _), _)| *id == obj_id).count();
                            verb_code_map.insert((obj_id, verb_count.to_string()), code);
                            temp_idx = new_idx;
                        } else {
                            break;
                        }
                    }
                    break;
                }
            }
            temp_idx += 1;
        }
        
        // Second pass: Parse objects with verb code available
        let mut objects = HashMap::new();
        
        // Reset to start of objects and parse with verb code
        while line_idx < lines.len() {
            let line = lines[line_idx].trim();
            
            // Object definitions are lines that contain ONLY #<number>
            if line.len() > 1 && line.starts_with('#') && line[1..].chars().all(|c| c.is_ascii_digit()) {
                match self.parse_object_full(&lines, line_idx, version, &verb_code_map) {
                    Ok(Some((obj, new_idx))) => {
                        objects.insert(obj.id, obj);
                        line_idx = new_idx;
                        continue;
                    }
                    Ok(None) => {
                        line_idx += 1;
                        continue;
                    }
                    Err(_) => {
                        // Skip this object if parsing fails
                        line_idx += 1;
                        continue;
                    }
                }
            }
            line_idx += 1;
            
            // Stop if we've reached the verb code section
            if line_idx > object_end_idx + 100 {
                break;
            }
        }

        // Post-process to resolve inherited property names
        self.resolve_inherited_property_names(&mut objects);
        
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

    fn parse_object_full(&self, lines: &[&str], start_idx: usize, _version: i32, verb_code_map: &HashMap<(i64, String), String>) -> Result<Option<(MooObject, usize)>> {
        if start_idx >= lines.len() {
            return Ok(None);
        }

        // Parse object ID
        let obj_line = lines[start_idx];
        let obj_id = match obj_line.strip_prefix('#').and_then(|s| s.parse::<i64>().ok()) {
            Some(id) => id,
            None => return Ok(None), // Not a valid object definition, skip it
        };

        let mut idx = start_idx + 1;
        
        // Check for recycled
        if idx < lines.len() && lines[idx].trim() == "recycled" {
            return Ok(Some((MooObject {
                id: obj_id,
                name: format!("Recycled Object #{}", obj_id),
                flags: 0,
                owner: -1,
                location: -1,
                parent: -1,
                children: Vec::new(),
                verbs: Vec::new(),
                properties: Vec::new(),
                defined_prop_count: 0,
                description: "This object has been recycled".to_string(),
            }, idx + 1)));
        }

        // Parse object name
        let name = if idx < lines.len() {
            lines[idx].to_string()
        } else {
            format!("Object #{}", obj_id)
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
                
                // Look up the actual verb code
                let code = verb_code_map.get(&(obj_id, verb_idx.to_string()))
                    .cloned()
                    .unwrap_or_else(|| "// Verb code not found".to_string());
                
                verbs.push(MooVerb {
                    name: verb_name,
                    owner: verb_owner,
                    permissions: format!("0x{:x}", verb_perms),
                    code,
                });
                idx += 4;
            } else {
                break;
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
            if idx < lines.len() {
                // Parse property value (simplified)
                // Properties beyond the defined ones are inherited
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
                    permissions: format!("0x{:x}", prop_perms),
                });
            }
        }

        Ok(Some((MooObject {
            id: obj_id,
            name,
            flags,
            owner,
            location,
            parent,
            children: Vec::new(), // Will be computed later
            verbs,
            properties,
            defined_prop_count: prop_def_count,
            description: format!("Object #{} with {} verbs and {} properties", obj_id, verb_count, prop_val_count),
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


    fn resolve_inherited_property_names(&self, objects: &mut HashMap<i64, MooObject>) {
        // First pass: Extract defined property names for each object
        let mut object_defined_props: HashMap<i64, Vec<String>> = HashMap::new();
        for (id, obj) in objects.iter() {
            // Use the defined_prop_count to know how many properties are defined on this object
            let mut defined_props = Vec::new();
            for i in 0..obj.defined_prop_count {
                if let Some(prop) = obj.properties.get(i) {
                    defined_props.push(prop.name.clone());
                }
            }
            object_defined_props.insert(*id, defined_props);
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
                // Start with parent's properties if there is one
                if obj.parent > 0 && obj.parent != obj_id && objects.contains_key(&obj.parent) {
                    all_props = build_property_list(obj.parent, objects, defined_props, cache);
                }
                
                // Add properties defined on this object
                if let Some(defined) = defined_props.get(&obj_id) {
                    all_props.extend(defined.iter().cloned());
                }
            }
            
            cache.insert(obj_id, all_props.clone());
            all_props
        }
        
        // Build complete property lists for all objects
        let obj_ids: Vec<i64> = objects.keys().cloned().collect();
        for obj_id in obj_ids {
            build_property_list(obj_id, objects, &object_defined_props, &mut complete_prop_names);
        }
        
        // Third pass: Update property names in objects
        for (id, obj) in objects.iter_mut() {
            if let Some(all_names) = complete_prop_names.get(id) {
                for (i, prop) in obj.properties.iter_mut().enumerate() {
                    if let Some(real_name) = all_names.get(i) {
                        // Determine if this property is inherited
                        let is_inherited = i >= obj.defined_prop_count;
                        
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
                    // Calculate current position
                    let total_props = obj.properties.len();
                    let total_verbs = obj.verbs.len();
                    
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
    
    fn navigate_middle_pane_down(&mut self) {
        if let Some(db) = self.current_database() {
            if let Some(obj_id) = self.selected_object {
                if let Some(obj) = db.objects.get(&obj_id) {
                    let total_props = obj.properties.len();
                    let total_verbs = obj.verbs.len();
                    
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
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();
    app.load_databases()?;
    app.update_selected_object();

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
    loop {
        terminal.draw(|f| ui(f, app))?;

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
                                                    let total_props = obj.properties.len();
                                                    // Account for headers and spacing (rough calculation)
                                                    if clicked_idx > 0 && clicked_idx <= total_props {
                                                        app.middle_pane_selection = MiddlePaneSelection::Property(clicked_idx - 1);
                                                    } else if clicked_idx > total_props + 2 && clicked_idx <= total_props + 2 + obj.verbs.len() {
                                                        app.middle_pane_selection = MiddlePaneSelection::Verb(clicked_idx - total_props - 3);
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
    for (i, prop) in obj.properties.iter().enumerate() {
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
    for (i, verb) in obj.verbs.iter().enumerate() {
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
        let mut items: Vec<ListItem> = Vec::new();
        let mut object_ids: Vec<i64> = db.objects.keys().copied().collect();
        object_ids.sort();

        for obj_id in object_ids {
            if let Some(obj) = db.objects.get(&obj_id) {
                let is_player = db.players.contains(&obj_id);
                let player_marker = if is_player { " [P]" } else { "" };
                
                let parent_info = if obj.parent >= 0 {
                    format!(" (parent: #{})", obj.parent)
                } else {
                    String::new()
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
                    obj_id, obj.name, player_marker, parent_info, obj.properties.len(), obj.verbs.len()
                )).style(style));
            }
        }

        let objects_list = List::new(items)
            .block(Block::default()
                .title(" Objects (Enter to view details) ")
                .borders(Borders::ALL))
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
        format!("Object #{}: {}", obj.id, obj.name),
        Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)
    )));
    details.push(Line::from(""));
    
    details.push(Line::from(Span::styled("Basic Information:", Style::default().add_modifier(Modifier::UNDERLINED))));
    details.push(Line::from(format!("ID: #{}", obj.id)));
    details.push(Line::from(format!("Name: {}", obj.name)));
    details.push(Line::from(format!("Flags: 0x{:x}", obj.flags)));
    details.push(Line::from(format!("Owner: #{}", obj.owner)));
    details.push(Line::from(format!("Location: #{}", obj.location)));
    
    if obj.parent >= 0 {
        let parent_name = db_objects.get(&obj.parent)
            .map(|p| p.name.as_str())
            .unwrap_or("Unknown");
        details.push(Line::from(format!("Parent: #{} ({})", obj.parent, parent_name)));
    } else {
        details.push(Line::from("Parent: None"));
    }

    if db_players.contains(&obj.id) {
        details.push(Line::from(Span::styled("Type: Player", Style::default().fg(Color::Green))));
    } else {
        details.push(Line::from("Type: Object"));
    }

    details.push(Line::from(""));
    details.push(Line::from(format!("Properties: {}", obj.properties.len())));
    details.push(Line::from(format!("Verbs: {}", obj.verbs.len())));
    details.push(Line::from(""));
    details.push(Line::from(Span::styled("Description:", Style::default().add_modifier(Modifier::UNDERLINED))));
    details.push(Line::from(obj.description.clone()));

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
        format!("Properties for #{}: {}", obj.id, obj.name),
        Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)
    )));
    details.push(Line::from(""));
    
    if obj.properties.is_empty() {
        details.push(Line::from("No properties defined."));
    } else {
        for (i, prop) in obj.properties.iter().enumerate() {
            details.push(Line::from(Span::styled(
                format!("Property #{}: {}", i + 1, prop.name),
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow)
            )));
            details.push(Line::from(format!("  Value: {}", prop.value)));
            details.push(Line::from(format!("  Owner: #{}", prop.owner)));
            details.push(Line::from(format!("  Permissions: {}", prop.permissions)));
            details.push(Line::from(""));
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
        format!("Verbs for #{}: {}", obj.id, obj.name),
        Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)
    )));
    details.push(Line::from(""));
    
    if obj.verbs.is_empty() {
        details.push(Line::from("No verbs defined."));
    } else {
        for (i, verb) in obj.verbs.iter().enumerate() {
            details.push(Line::from(Span::styled(
                format!("Verb #{}: {}", i + 1, verb.name),
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Green)
            )));
            details.push(Line::from(format!("  Owner: #{}", verb.owner)));
            details.push(Line::from(format!("  Permissions: {}", verb.permissions)));
            
            // Try to get actual code from the verb_code_map
            let code = db_verb_code_map.get(&(obj.id, verb.name.clone()))
                .cloned()
                .unwrap_or_else(|| verb.code.clone());
            
            // Pretty-print the code
            let pretty_code = pretty_print_moo_code(&code);
            
            details.push(Line::from("  Code:"));
            for line in pretty_code.lines() {
                details.push(Line::from(format!("    {}", line)));
            }
            details.push(Line::from(""));
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
        format!("Relationships for #{}: {}", obj.id, obj.name),
        Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)
    )));
    details.push(Line::from(""));
    
    // Parent
    if obj.parent >= 0 {
        if let Some(parent_obj) = db_objects.get(&obj.parent) {
            details.push(Line::from(Span::styled("Parent:", Style::default().add_modifier(Modifier::UNDERLINED))));
            details.push(Line::from(format!("  #{} - {}", obj.parent, parent_obj.name)));
        }
    } else {
        details.push(Line::from(Span::styled("Parent:", Style::default().add_modifier(Modifier::UNDERLINED))));
        details.push(Line::from("  None (root object)"));
    }
    details.push(Line::from(""));
    
    // Children
    let mut children: Vec<i64> = db_objects.values()
        .filter(|o| o.parent == obj.id)
        .map(|o| o.id)
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
                details.push(Line::from(format!("  #{} - {}", child_id, child_obj.name)));
            }
        }
    }
    details.push(Line::from(""));
    
    // Location
    if obj.location >= 0 {
        if let Some(location_obj) = db_objects.get(&obj.location) {
            details.push(Line::from(Span::styled("Location:", Style::default().add_modifier(Modifier::UNDERLINED))));
            details.push(Line::from(format!("  #{} - {}", obj.location, location_obj.name)));
            details.push(Line::from(""));
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
    
    if let Some(prop) = obj.properties.get(prop_idx) {
        details.push(Line::from(Span::styled(
            format!("Property: {}", prop.name),
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow)
        )));
        details.push(Line::from(""));
        
        details.push(Line::from(format!("Value: {}", prop.value)));
        details.push(Line::from(format!("Owner: #{}", prop.owner)));
        details.push(Line::from(format!("Permissions: {}", prop.permissions)));
        
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
    
    if let Some(verb) = obj.verbs.get(verb_idx) {
        details.push(Line::from(Span::styled(
            format!("Verb: {}", verb.name),
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Green)
        )));
        details.push(Line::from(""));
        
        details.push(Line::from(format!("Owner: #{}", verb.owner)));
        details.push(Line::from(format!("Permissions: {}", verb.permissions)));
        details.push(Line::from(""));
        
        // Get actual code from the verb_code_map
        let code = verb_code_map.get(&(obj.id, verb.name.clone()))
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

                items.push(ListItem::new(format!(
                    "#{:3} {}{}", 
                    obj_id, 
                    if obj.name.len() > 20 { 
                        format!("{}...", &obj.name[..17])
                    } else {
                        obj.name.clone()
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