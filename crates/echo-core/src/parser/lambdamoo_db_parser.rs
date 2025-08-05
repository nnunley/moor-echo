use std::collections::HashMap;

use anyhow::{anyhow, Result};
use pest::Parser;
use pest_derive::Parser;

use crate::ast::EchoAst;
use crate::storage::{ObjectId, PropertyValue};

#[derive(Parser)]
#[grammar = "parser/lambdamoo_db_grammar.pest"]
pub struct LambdaMooDbParser;

/// Represents a LambdaMOO database file
pub struct LambdaMooDatabase {
    pub version: i32,
    pub total_objects: i64,
    pub total_verbs: i64,
    pub total_players: i64,
    pub player_list: Vec<i64>,
    pub objects: HashMap<i64, LambdaMooObject>,
    pub verb_programs: HashMap<(i64, String), String>, // (objid, verb_name) -> code
}

/// Represents a single object in the database
pub struct LambdaMooObject {
    pub id: i64,
    pub name: String,
    pub flags: i64,
    pub owner: i64,
    pub location: i64,
    pub contents: i64,
    pub next: i64,
    pub parent: i64,
    pub child: i64,
    pub sibling: i64,
    pub verbs: Vec<LambdaMooVerb>,
    pub properties: Vec<LambdaMooProperty>,
    pub property_values: Vec<LambdaMooPropertyValue>,
    pub is_recycled: bool,
}

/// Represents a verb definition
pub struct LambdaMooVerb {
    pub name: String,
    pub owner: i64,
    pub perms: i64,
    pub prep: i64,
}

/// Represents a property definition
pub struct LambdaMooProperty {
    pub name: String,
}

/// Represents a property value
pub struct LambdaMooPropertyValue {
    pub value: LambdaMooValue,
    pub owner: i64,
    pub perms: i64,
}

/// Represents a MOO value
#[derive(Debug, Clone)]
pub enum LambdaMooValue {
    Clear,
    None,
    Str(String),
    Obj(i64),
    Err(i64),
    Int(i64),
    Float(f64),
    List(Vec<LambdaMooValue>),
    Map(Vec<(LambdaMooValue, LambdaMooValue)>),
}

// Value type constants from LambdaMOO
const TYPE_CLEAR: i64 = -2;
const TYPE_NONE: i64 = -1;
const TYPE_STR: i64 = 0;
const TYPE_OBJ: i64 = 1;
const TYPE_ERR: i64 = 2;
const TYPE_INT: i64 = 3;
const TYPE_CATCH: i64 = 4;
const TYPE_FINALLY: i64 = 5;
const TYPE_FLOAT: i64 = 9;
const TYPE_LIST: i64 = 10;
const TYPE_MAP: i64 = 12;

impl LambdaMooDbParser {
    /// Parse a LambdaMOO database file
    pub fn parse_database(input: &str) -> Result<LambdaMooDatabase> {
        let pairs = LambdaMooDbParser::parse(Rule::database, input)
            .map_err(|e| anyhow!("Parse error: {}", e))?;
        
        let mut db = LambdaMooDatabase {
            version: 0,
            total_objects: 0,
            total_verbs: 0,
            total_players: 0,
            player_list: Vec::new(),
            objects: HashMap::new(),
            verb_programs: HashMap::new(),
        };
        
        for pair in pairs {
            match pair.as_rule() {
                Rule::database => {
                    Self::parse_database_content(pair, &mut db)?;
                }
                _ => {}
            }
        }
        
        Ok(db)
    }
    
    fn parse_database_content(pair: pest::iterators::Pair<Rule>, db: &mut LambdaMooDatabase) -> Result<()> {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::header => {
                    db.version = Self::parse_header(inner)?;
                }
                Rule::intro_block => {
                    Self::parse_intro_block(inner, db)?;
                }
                Rule::player_list => {
                    db.player_list = Self::parse_player_list(inner)?;
                }
                Rule::object_list => {
                    Self::parse_object_list(inner, db)?;
                }
                Rule::verb_programs => {
                    Self::parse_verb_programs(inner, db)?;
                }
                Rule::final_tasks_section => {
                    Self::parse_final_tasks_section(inner)?;
                }
                Rule::clocks_section => {
                    Self::parse_clocks_section(inner)?;
                }
                Rule::queued_tasks_section => {
                    Self::parse_queued_tasks_section(inner)?;
                }
                Rule::suspended_tasks_section => {
                    Self::parse_suspended_tasks_section(inner)?;
                }
                Rule::finalization_section => {
                    Self::parse_finalization_section(inner)?;
                }
                _ => {
                    // Skip unknown rules
                }
            }
        }
        Ok(())
    }
    
    fn parse_header(pair: pest::iterators::Pair<Rule>) -> Result<i32> {
        for inner in pair.into_inner() {
            if let Rule::db_version = inner.as_rule() {
                return Ok(inner.as_str().parse()?);
            }
        }
        Err(anyhow!("No database version found in header"))
    }
    
    fn parse_intro_block(pair: pest::iterators::Pair<Rule>, db: &mut LambdaMooDatabase) -> Result<()> {
        let mut inner = pair.into_inner();
        
        if let Some(total_objects) = inner.next() {
            db.total_objects = total_objects.as_str().parse()?;
        }
        if let Some(total_verbs) = inner.next() {
            db.total_verbs = total_verbs.as_str().parse()?;
        }
        if let Some(_dummy) = inner.next() {
            // Skip dummy value
        }
        if let Some(total_players) = inner.next() {
            db.total_players = total_players.as_str().parse()?;
        }
        
        Ok(())
    }
    
    fn parse_player_list(pair: pest::iterators::Pair<Rule>) -> Result<Vec<i64>> {
        let mut players = Vec::new();
        for inner in pair.into_inner() {
            if let Rule::player_id = inner.as_rule() {
                players.push(inner.as_str().parse()?);
            }
        }
        Ok(players)
    }
    
    fn parse_object_list(pair: pest::iterators::Pair<Rule>, db: &mut LambdaMooDatabase) -> Result<()> {
        let mut object_count = 0;
        let inner = pair.into_inner();
        
        let inner_items: Vec<_> = inner.collect();
        
        if inner_items.is_empty() {
            return Ok(());
        }
        
        let mut inner = inner_items.into_iter();
        
        // Check if first item is object count or object definition
        if let Some(first_pair) = inner.next() {
            match first_pair.as_rule() {
                Rule::object_count => {
                    let _expected_count: i64 = first_pair.as_str().parse()?;
                    
                    // Parse remaining object definitions
                    for object_def in inner {
                        if let Rule::object_def = object_def.as_rule() {
                            let obj = Self::parse_object(object_def)?;
                            db.objects.insert(obj.id, obj);
                            object_count += 1;
                        }
                    }
                }
                Rule::object_def => {
                    // Parse first object
                    let obj = Self::parse_object(first_pair)?;
                    db.objects.insert(obj.id, obj);
                    object_count += 1;
                    
                    // Parse remaining objects
                    for object_def in inner {
                        if let Rule::object_def = object_def.as_rule() {
                            let obj = Self::parse_object(object_def)?;
                            db.objects.insert(obj.id, obj);
                            object_count += 1;
                        }
                    }
                }
                _ => {
                }
            }
        }
        
        Ok(())
    }
    
    fn parse_object(pair: pest::iterators::Pair<Rule>) -> Result<LambdaMooObject> {
        let mut obj = LambdaMooObject {
            id: 0,
            name: String::new(),
            flags: 0,
            owner: 0,
            location: 0,
            contents: 0,
            next: 0,
            parent: 0,
            child: 0,
            sibling: 0,
            verbs: Vec::new(),
            properties: Vec::new(),
            property_values: Vec::new(),
            is_recycled: false,
        };
        
        let mut inner = pair.into_inner();
        
        // Parse object header
        if let Some(header) = inner.next() {
            for header_inner in header.into_inner() {
                if let Rule::objid = header_inner.as_rule() {
                    obj.id = header_inner.as_str().parse()?;
                }
            }
        }
        
        // Check if recycled or parse body
        if let Some(next) = inner.next() {
            match next.as_rule() {
                Rule::recycled_marker => {
                    obj.is_recycled = true;
                }
                Rule::object_body => {
                    Self::parse_object_body(next, &mut obj)?;
                }
                _ => {}
            }
        }
        
        Ok(obj)
    }
    
    fn parse_object_body(pair: pest::iterators::Pair<Rule>, obj: &mut LambdaMooObject) -> Result<()> {
        let mut inner = pair.into_inner();
        
        // Check which object body format we have
        if let Some(body_format) = inner.next() {
            match body_format.as_rule() {
                Rule::lambdamoo_object_body => {
                    Self::parse_lambdamoo_object_body(body_format, obj)?;
                }
                Rule::toaststunt_object_body => {
                    Self::parse_toaststunt_object_body(body_format, obj)?;
                }
                _ => {
                }
            }
        }
        
        Ok(())
    }
    
    fn parse_lambdamoo_object_body(pair: pest::iterators::Pair<Rule>, obj: &mut LambdaMooObject) -> Result<()> {
        let mut inner = pair.into_inner();
        
        if let Some(name) = inner.next() {
            obj.name = name.as_str().to_string();
        }
        if let Some(_handles) = inner.next() {
            // Skip old handles string
        }
        if let Some(flags) = inner.next() {
            obj.flags = flags.as_str().parse()
                .map_err(|e| anyhow::anyhow!("Failed to parse flags '{}': {}", flags.as_str(), e))?;
        }
        if let Some(owner) = inner.next() {
            obj.owner = owner.as_str().parse()
                .map_err(|e| anyhow::anyhow!("Failed to parse owner '{}': {}", owner.as_str(), e))?;
        }
        if let Some(location) = inner.next() {
            obj.location = location.as_str().parse()
                .map_err(|e| anyhow::anyhow!("Failed to parse location '{}': {}", location.as_str(), e))?;
        }
        if let Some(contents) = inner.next() {
            obj.contents = contents.as_str().parse()
                .map_err(|e| anyhow::anyhow!("Failed to parse contents '{}': {}", contents.as_str(), e))?;
        }
        if let Some(next) = inner.next() {
            obj.next = next.as_str().parse()
                .map_err(|e| anyhow::anyhow!("Failed to parse next '{}': {}", next.as_str(), e))?;
        }
        if let Some(parent) = inner.next() {
            obj.parent = parent.as_str().parse()
                .map_err(|e| anyhow::anyhow!("Failed to parse parent '{}': {}", parent.as_str(), e))?;
        }
        if let Some(child) = inner.next() {
            obj.child = child.as_str().parse()
                .map_err(|e| anyhow::anyhow!("Failed to parse child '{}': {}", child.as_str(), e))?;
        }
        if let Some(sibling) = inner.next() {
            obj.sibling = sibling.as_str().parse()
                .map_err(|e| anyhow::anyhow!("Failed to parse sibling '{}': {}", sibling.as_str(), e))?;
        }
        
        // Parse standard verb definitions
        if let Some(verb_defs) = inner.next() {
            obj.verbs = Self::parse_verb_definitions(verb_defs)?;
        }
        
        // Parse property definitions
        if let Some(prop_defs) = inner.next() {
            obj.properties = Self::parse_property_definitions(prop_defs)?;
        }
        
        // Parse property values
        if let Some(prop_vals) = inner.next() {
            obj.property_values = Self::parse_property_values(prop_vals)?;
        }
        
        Ok(())
    }
    
    fn parse_toaststunt_object_body(pair: pest::iterators::Pair<Rule>, obj: &mut LambdaMooObject) -> Result<()> {
        let mut inner = pair.into_inner();
        
        if let Some(name) = inner.next() {
            obj.name = name.as_str().to_string();
        }
        if let Some(_handles) = inner.next() {
            // Skip old handles string (read and discarded in ToastStunt)
        }
        if let Some(flags) = inner.next() {
            obj.flags = flags.as_str().parse()?;
        }
        if let Some(owner) = inner.next() {
            obj.owner = owner.as_str().parse()?;
        }
        
        // ToastStunt variable fields - we'll parse the first simple value for now
        if let Some(location_var) = inner.next() {
            // For now, just get the first number from the location variable
            if let Some(first_val) = location_var.into_inner().next() {
                obj.location = first_val.as_str().parse().unwrap_or(-1);
            }
        }
        if let Some(_last_move_var) = inner.next() {
            // Skip last_move for now
        }
        if let Some(contents_var) = inner.next() {
            // For now, just get the first number from the contents variable
            if let Some(first_val) = contents_var.into_inner().next() {
                obj.contents = first_val.as_str().parse().unwrap_or(0);
            }
        }
        if let Some(parents_var) = inner.next() {
            // For now, just get the first number as parent
            if let Some(first_val) = parents_var.into_inner().next() {
                obj.parent = first_val.as_str().parse().unwrap_or(0);
            }
        }
        if let Some(_children_var) = inner.next() {
            // Skip children for now - set child to 0
            obj.child = 0;
        }
        
        // Set remaining fields to default values for ToastStunt
        obj.next = 0;
        obj.sibling = 0;
        
        // Parse extended verb definitions
        if let Some(verb_defs_ext) = inner.next() {
            obj.verbs = Self::parse_verb_definitions_extended(verb_defs_ext)?;
        }
        
        // Parse property definitions
        if let Some(prop_defs) = inner.next() {
            obj.properties = Self::parse_property_definitions(prop_defs)?;
        }
        
        // Parse property values
        if let Some(prop_vals) = inner.next() {
            obj.property_values = Self::parse_property_values(prop_vals)?;
        }
        
        Ok(())
    }
    
    fn parse_verb_definitions(pair: pest::iterators::Pair<Rule>) -> Result<Vec<LambdaMooVerb>> {
        let mut verbs = Vec::new();
        let mut inner = pair.into_inner();
        
        // Skip verb count
        let _ = inner.next();
        
        // Parse each verb
        for verb_def in inner {
            if let Rule::verb_def = verb_def.as_rule() {
                verbs.push(Self::parse_verb_def(verb_def)?);
            }
        }
        
        Ok(verbs)
    }
    
    fn parse_verb_definitions_extended(pair: pest::iterators::Pair<Rule>) -> Result<Vec<LambdaMooVerb>> {
        let mut verbs = Vec::new();
        let mut inner = pair.into_inner();
        
        // Skip verb count
        if let Some(verb_count) = inner.next() {
            let _count: i64 = verb_count.as_str().parse()?;
        }
        
        // Check if next item is ToastStunt extensions or verb definition
        if let Some(next_item) = inner.next() {
            match next_item.as_rule() {
                Rule::toaststunt_verb_extensions => {
                    // Skip the extensions and parse remaining verb definitions
                    for verb_def in inner {
                        if let Rule::verb_def = verb_def.as_rule() {
                            verbs.push(Self::parse_verb_def(verb_def)?);
                        }
                    }
                }
                Rule::verb_def => {
                    // No extensions, parse this and remaining verb definitions
                    verbs.push(Self::parse_verb_def(next_item)?);
                    for verb_def in inner {
                        if let Rule::verb_def = verb_def.as_rule() {
                            verbs.push(Self::parse_verb_def(verb_def)?);
                        }
                    }
                }
                _ => {
                }
            }
        }
        
        Ok(verbs)
    }
    
    fn parse_verb_def(pair: pest::iterators::Pair<Rule>) -> Result<LambdaMooVerb> {
        let mut verb = LambdaMooVerb {
            name: String::new(),
            owner: 0,
            perms: 0,
            prep: 0,
        };
        
        let mut inner = pair.into_inner();
        
        if let Some(name) = inner.next() {
            verb.name = name.as_str().to_string();
        }
        if let Some(owner) = inner.next() {
            verb.owner = owner.as_str().parse()?;
        }
        if let Some(perms) = inner.next() {
            verb.perms = perms.as_str().parse()?;
        }
        if let Some(prep) = inner.next() {
            verb.prep = prep.as_str().parse()?;
        }
        
        Ok(verb)
    }
    
    fn parse_property_definitions(pair: pest::iterators::Pair<Rule>) -> Result<Vec<LambdaMooProperty>> {
        let mut props = Vec::new();
        let mut inner = pair.into_inner();
        
        // Skip property count
        let _ = inner.next();
        
        // Parse each property
        for prop_def in inner {
            if let Rule::propdef = prop_def.as_rule() {
                let mut prop_inner = prop_def.into_inner();
                if let Some(name) = prop_inner.next() {
                    props.push(LambdaMooProperty {
                        name: name.as_str().to_string(),
                    });
                }
            }
        }
        
        Ok(props)
    }
    
    fn parse_property_values(pair: pest::iterators::Pair<Rule>) -> Result<Vec<LambdaMooPropertyValue>> {
        let mut values = Vec::new();
        let mut inner = pair.into_inner();
        
        // Skip property value count
        let _ = inner.next();
        
        // Parse each property value
        for prop_val in inner {
            if let Rule::propval = prop_val.as_rule() {
                values.push(Self::parse_propval(prop_val)?);
            }
        }
        
        Ok(values)
    }
    
    fn parse_propval(pair: pest::iterators::Pair<Rule>) -> Result<LambdaMooPropertyValue> {
        let mut value = LambdaMooPropertyValue {
            value: LambdaMooValue::None,
            owner: 0,
            perms: 0,
        };
        
        let mut inner = pair.into_inner();
        
        if let Some(val) = inner.next() {
            value.value = Self::parse_value(val)?;
        }
        if let Some(owner) = inner.next() {
            value.owner = owner.as_str().parse()?;
        }
        if let Some(perms) = inner.next() {
            value.perms = perms.as_str().parse()?;
        }
        
        Ok(value)
    }
    
    fn parse_value(pair: pest::iterators::Pair<Rule>) -> Result<LambdaMooValue> {
        let mut inner = pair.into_inner();
        
        // Get value type
        let value_type = if let Some(vtype) = inner.next() {
            vtype.as_str().parse::<i64>()
                .map_err(|e| anyhow::anyhow!("Failed to parse value type '{}': {}", vtype.as_str(), e))?
        } else {
            return Err(anyhow!("Missing value type"));
        };
        
        // Parse value content based on type
        match value_type {
            TYPE_CLEAR => Ok(LambdaMooValue::Clear),
            TYPE_NONE => Ok(LambdaMooValue::None),
            _ => if let Some(content) = inner.next() {
                match value_type {
                TYPE_STR => {
                    let mut content_inner = content.into_inner();
                    if let Some(str_val) = content_inner.next() {
                        Ok(LambdaMooValue::Str(str_val.as_str().to_string()))
                    } else {
                        Ok(LambdaMooValue::Str(String::new()))
                    }
                }
                TYPE_OBJ => {
                    let mut content_inner = content.into_inner();
                    if let Some(obj_val) = content_inner.next() {
                        // obj_val is the obj_value rule, we need to extract the objid part
                        let mut obj_inner = obj_val.into_inner();
                        if let Some(objid_val) = obj_inner.next() {
                            let obj_str = objid_val.as_str();
                            if obj_str.trim().is_empty() {
                                Ok(LambdaMooValue::Obj(-1)) // Default object ID for empty values (invalid object)
                            } else {
                                Ok(LambdaMooValue::Obj(obj_str.parse()
                                    .map_err(|e| anyhow::anyhow!("Failed to parse object ID '{}': {}", obj_str, e))?))
                            }
                        } else {
                            Ok(LambdaMooValue::Obj(-1)) // Default when no object ID present
                        }
                    } else {
                        Err(anyhow!("Missing object ID"))
                    }
                }
                TYPE_ERR => {
                    let mut content_inner = content.into_inner();
                    if let Some(err_val) = content_inner.next() {
                        // err_val is the err_value rule, we need to extract the num part
                        let mut err_inner = err_val.into_inner();
                        if let Some(num_val) = err_inner.next() {
                            let num_str = num_val.as_str();
                            if num_str.trim().is_empty() {
                                Ok(LambdaMooValue::Err(0)) // Default error code for empty values
                            } else {
                                Ok(LambdaMooValue::Err(num_str.parse()
                                    .map_err(|e| anyhow::anyhow!("Failed to parse error code '{}': {}", num_str, e))?))
                            }
                        } else {
                            Ok(LambdaMooValue::Err(0)) // Default when no error code present
                        }
                    } else {
                        Err(anyhow!("Missing error code"))
                    }
                }
                TYPE_INT | TYPE_CATCH | TYPE_FINALLY => {
                    let mut content_inner = content.into_inner();
                    if let Some(int_val) = content_inner.next() {
                        // int_val is the int_value rule, we need to extract the num part
                        let mut int_inner = int_val.into_inner();
                        if let Some(num_val) = int_inner.next() {
                            let num_str = num_val.as_str();
                            if num_str.trim().is_empty() {
                                Ok(LambdaMooValue::Int(0)) // Default integer for empty values
                            } else {
                                Ok(LambdaMooValue::Int(num_str.parse()
                                    .map_err(|e| anyhow::anyhow!("Failed to parse integer value '{}': {}", num_str, e))?))
                            }
                        } else {
                            Ok(LambdaMooValue::Int(0)) // Default when no integer present
                        }
                    } else {
                        Err(anyhow!("Missing integer value"))
                    }
                }
                TYPE_FLOAT => {
                    let mut content_inner = content.into_inner();
                    if let Some(float_val) = content_inner.next() {
                        // float_val is the float_value rule, we need to extract the float_num part
                        let mut float_inner = float_val.into_inner();
                        if let Some(num_val) = float_inner.next() {
                            let num_str = num_val.as_str();
                            if num_str.trim().is_empty() {
                                Ok(LambdaMooValue::Float(0.0)) // Default float for empty values
                            } else {
                                Ok(LambdaMooValue::Float(num_str.parse()
                                    .map_err(|e| anyhow::anyhow!("Failed to parse float value '{}': {}", num_str, e))?))
                            }
                        } else {
                            Ok(LambdaMooValue::Float(0.0)) // Default when no float present
                        }
                    } else {
                        Err(anyhow!("Missing float value"))
                    }
                }
                TYPE_LIST => {
                    Self::parse_list_value(content)
                }
                TYPE_MAP => {
                    Self::parse_map_value(content)
                }
                _ => Err(anyhow!("Unknown value type: {}", value_type)),
                }
            } else {
                Err(anyhow!("Missing value content for type {}", value_type))
            }
        }
    }
    
    fn parse_list_value(pair: pest::iterators::Pair<Rule>) -> Result<LambdaMooValue> {
        let mut list = Vec::new();
        let mut inner = pair.into_inner();
        
        // Skip list length  
        if let Some(len_pair) = inner.next() {
            let list_len_str = len_pair.as_str();
            let _list_len = list_len_str.parse::<i64>()
                .map_err(|e| anyhow::anyhow!("Failed to parse list length '{}': {}", list_len_str, e))?;
        }
        
        // Parse each list element
        let mut elem_count = 0;
        for elem in inner {
            if let Rule::value = elem.as_rule() {
                elem_count += 1;
                list.push(Self::parse_value(elem)
                    .map_err(|e| anyhow::anyhow!("Failed to parse list element {}: {}", elem_count, e))?);
            }
        }
        
        Ok(LambdaMooValue::List(list))
    }
    
    fn parse_map_value(pair: pest::iterators::Pair<Rule>) -> Result<LambdaMooValue> {
        let mut map = Vec::new();
        let mut inner = pair.into_inner();
        
        // Skip map length
        let _ = inner.next();
        
        // Parse each map entry
        for entry in inner {
            if let Rule::map_entry = entry.as_rule() {
                let mut entry_inner = entry.into_inner();
                if let (Some(key), Some(value)) = (entry_inner.next(), entry_inner.next()) {
                    map.push((Self::parse_value(key)?, Self::parse_value(value)?));
                }
            }
        }
        
        Ok(LambdaMooValue::Map(map))
    }
    
    fn parse_verb_programs(pair: pest::iterators::Pair<Rule>, db: &mut LambdaMooDatabase) -> Result<()> {
        for section in pair.into_inner() {
            match section.as_rule() {
                Rule::verb_code_section => {
                    Self::parse_verb_code_section(section, db)?;
                }
                // TODO: Parse other sections (clocks, tasks) if needed
                _ => {}
            }
        }
        Ok(())
    }
    
    fn parse_final_tasks_section(_pair: pest::iterators::Pair<Rule>) -> Result<()> {
        // Just skip the final tasks counts for now
        Ok(())
    }
    
    fn parse_clocks_section(_pair: pest::iterators::Pair<Rule>) -> Result<()> {
        // Skip for now
        Ok(())
    }
    
    fn parse_queued_tasks_section(_pair: pest::iterators::Pair<Rule>) -> Result<()> {
        // Skip for now
        Ok(())
    }
    
    fn parse_suspended_tasks_section(_pair: pest::iterators::Pair<Rule>) -> Result<()> {
        // Skip for now
        Ok(())
    }
    
    fn parse_finalization_section(_pair: pest::iterators::Pair<Rule>) -> Result<()> {
        // Skip for now
        Ok(())
    }
    
    fn parse_verb_code_section(pair: pest::iterators::Pair<Rule>, db: &mut LambdaMooDatabase) -> Result<()> {
        let mut inner = pair.into_inner();
        
        // Skip total verb count
        let _ = inner.next();
        
        // Parse each verb program
        for verb_prog in inner {
            if let Rule::verb_program = verb_prog.as_rule() {
                Self::parse_verb_program(verb_prog, db)?;
            }
        }
        
        Ok(())
    }
    
    fn parse_verb_program(pair: pest::iterators::Pair<Rule>, db: &mut LambdaMooDatabase) -> Result<()> {
        let mut inner = pair.into_inner();
        
        // Parse verb header (format: "#objid:verbname")
        if let Some(header) = inner.next() {
            let header_str = header.as_str();
            if let Some(colon_pos) = header_str.find(':') {
                let objid_str = &header_str[1..colon_pos]; // Skip '#'
                let verb_name = &header_str[colon_pos + 1..];
                let objid: i64 = objid_str.parse()?;
                
                // Parse program code
                if let Some(code) = inner.next() {
                    let program_lines: Vec<String> = code.into_inner()
                        .filter(|p| p.as_rule() == Rule::program_line)
                        .map(|p| p.as_str().to_string())
                        .collect();
                    
                    let program_text = program_lines.join("\n");
                    db.verb_programs.insert((objid, verb_name.to_string()), program_text);
                }
            }
        }
        
        Ok(())
    }
}

/// Convert LambdaMOO value to Echo AST
impl LambdaMooValue {
    pub fn to_echo_ast(&self) -> EchoAst {
        match self {
            LambdaMooValue::Clear | LambdaMooValue::None => EchoAst::Null,
            LambdaMooValue::Str(s) => EchoAst::String(s.clone()),
            LambdaMooValue::Obj(id) => EchoAst::ObjectRef(*id),
            LambdaMooValue::Err(code) => EchoAst::Number(*code), // Errors as numbers for now
            LambdaMooValue::Int(n) => EchoAst::Number(*n),
            LambdaMooValue::Float(f) => EchoAst::Float(*f),
            LambdaMooValue::List(items) => {
                let echo_items: Vec<EchoAst> = items.iter()
                    .map(|v| v.to_echo_ast())
                    .collect();
                EchoAst::List { elements: echo_items }
            }
            LambdaMooValue::Map(entries) => {
                let echo_entries: Vec<(String, EchoAst)> = entries.iter()
                    .enumerate()
                    .map(|(i, (_k, v))| (format!("key_{}", i), v.to_echo_ast()))
                    .collect();
                EchoAst::Map { entries: echo_entries }
            }
        }
    }
    
    pub fn to_property_value(&self) -> PropertyValue {
        match self {
            LambdaMooValue::Clear | LambdaMooValue::None => PropertyValue::Null,
            LambdaMooValue::Str(s) => PropertyValue::String(s.clone()),
            LambdaMooValue::Obj(_id) => PropertyValue::Object(ObjectId::new()), // Need ID mapping
            LambdaMooValue::Err(code) => PropertyValue::Integer(*code),
            LambdaMooValue::Int(n) => PropertyValue::Integer(*n),
            LambdaMooValue::Float(f) => PropertyValue::Float(*f),
            LambdaMooValue::List(_) => PropertyValue::List(Vec::new()), // TODO: Implement list conversion
            LambdaMooValue::Map(_) => PropertyValue::Map(HashMap::new()), // TODO: Implement map conversion
        }
    }
}