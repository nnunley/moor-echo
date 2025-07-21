//! Echo Runtime - High-level interface for Echo language execution
//! 
//! Provides a simplified interface that wraps the evaluator and storage
//! for use by external components like REPL and web interfaces.

use anyhow::Result;
use std::sync::Arc;

use crate::{
    EchoConfig, EchoError,
    evaluator::{Evaluator, Value},
    parser::{Parser, create_parser},
    storage::{Storage, ObjectId},
    ui_callback::{UiEventCallback, UiEvent, UiAction, convert_ui_event},
};

/// High-level Echo runtime that combines parser, evaluator, and storage
pub struct EchoRuntime {
    evaluator: Evaluator,
    parser: Box<dyn Parser>,
    _storage: Arc<Storage>,
    ui_callback: Option<UiEventCallback>,
}

impl EchoRuntime {
    /// Create a new Echo runtime with the given configuration
    pub fn new(config: EchoConfig) -> Result<Self> {
        // Initialize storage
        let storage = Arc::new(Storage::new(&config.storage_path)?);
        
        // Create evaluator
        let evaluator = Evaluator::new(storage.clone());
        
        // Create parser
        let parser = create_parser("echo")?;
        
        Ok(Self {
            evaluator,
            parser,
            _storage: storage,
            ui_callback: None,
        })
    }
    
    /// Evaluate Echo source code and return the result
    pub fn eval_source(&mut self, source: &str) -> Result<Value> {
        // Parse the source code
        let ast = self.parser.parse(source)?;
        
        // Ensure we have a player (create default if needed)
        if self.evaluator.current_player().is_none() {
            // Try to find existing default player first
            let player_id = match self.find_player_by_name("default") {
                Ok(existing_id) => existing_id,
                Err(_) => {
                    // Create a default player if none exists
                    self.evaluator.create_player("default")?
                }
            };
            self.evaluator.switch_player(player_id)?;
        }
        
        // Evaluate the AST
        self.evaluator.eval(&ast)
    }
    
    /// Create a new player
    pub fn create_player(&mut self, name: &str) -> Result<ObjectId> {
        self.evaluator.create_player(name)
    }
    
    /// Switch to an existing player
    pub fn switch_player(&mut self, player_id: ObjectId) -> Result<()> {
        self.evaluator.switch_player(player_id)
    }
    
    /// Get the current player ID
    pub fn current_player(&self) -> Option<ObjectId> {
        self.evaluator.current_player()
    }
    
    /// Find a player by name and return their ID
    pub fn find_player_by_name(&self, name: &str) -> Result<ObjectId> {
        // Access the storage to find the player by name
        use crate::storage::PropertyValue;
        
        let system_obj = self._storage.objects.get(ObjectId::system())?;
        
        if let Some(PropertyValue::Map(player_registry)) = system_obj.properties.get("player_registry") {
            if let Some(PropertyValue::Object(player_id)) = player_registry.get(name) {
                return Ok(*player_id);
            }
        }
        
        Err(anyhow::anyhow!("Player '{}' not found", name))
    }
    
    /// Switch to a player by name
    pub fn switch_player_by_name(&mut self, name: &str) -> Result<()> {
        let player_id = self.find_player_by_name(name)?;
        self.switch_player(player_id)
    }
    
    /// List all players (returns a simplified representation)
    pub fn list_players(&self) -> Result<Vec<(String, ObjectId)>> {
        use crate::storage::PropertyValue;
        
        let system_obj = self._storage.objects.get(ObjectId::system())?;
        let mut players = Vec::new();
        
        if let Some(PropertyValue::Map(player_registry)) = system_obj.properties.get("player_registry") {
            for (name, value) in player_registry {
                if let PropertyValue::Object(player_id) = value {
                    players.push((name.clone(), *player_id));
                }
            }
        }
        
        Ok(players)
    }
    
    /// Get a mutable reference to the parser
    pub fn parser_mut(&mut self) -> &mut Box<dyn Parser> {
        &mut self.parser
    }
    
    /// Parse Echo program (multi-statement) into an AST
    pub fn parse_program(&mut self, source: &str) -> Result<crate::ast::EchoAst> {
        self.parser.parse_program(source)
    }
    
    /// Evaluate an Echo AST directly
    pub fn eval(&mut self, ast: &crate::ast::EchoAst) -> Result<Value> {
        // Ensure we have a player (create default if needed)
        if self.evaluator.current_player().is_none() {
            // Try to find existing default player first
            let player_id = match self.find_player_by_name("default") {
                Ok(existing_id) => existing_id,
                Err(_) => {
                    // Create a default player if none exists
                    self.evaluator.create_player("default")?
                }
            };
            self.evaluator.switch_player(player_id)?;
        }
        
        // Evaluate the AST
        let result = self.evaluator.eval(ast);
        
        // Check for UI events after evaluation
        self.check_ui_events();
        
        result
    }
    
    /// Set a UI event callback
    pub fn set_ui_callback(&mut self, callback: UiEventCallback) {
        // Set on both runtime and evaluator
        self.ui_callback = Some(callback.clone());
        self.evaluator.set_ui_callback(callback);
    }
    
    /// Check for UI events and forward them to the callback
    fn check_ui_events(&mut self) {
        if let Some(ref callback) = self.ui_callback {
            // This is a simplified approach - in a real implementation,
            // we'd need to properly integrate with the event system
            // For now, we'll simulate UI events for testing
        }
    }
    
    /// Get a mutable reference to the evaluator
    pub fn evaluator_mut(&mut self) -> &mut Evaluator {
        &mut self.evaluator
    }
    
    /// Get the current environment variables for the current player
    pub fn get_environment_vars(&self) -> Vec<(String, crate::evaluator::Value)> {
        if let Some(player_id) = self.evaluator.current_player() {
            self.evaluator.get_player_environment(player_id)
                .map(|vars| vars.into_iter().collect())
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    }
}