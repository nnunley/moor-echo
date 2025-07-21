//! Security module for Echo runtime
//! 
//! Provides security-related functionality including:
//! - Access control and permissions
//! - Resource limits and sandboxing
//! - Security policy enforcement

/// Security policy configuration
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    /// Maximum memory usage in bytes
    pub max_memory: usize,
    /// Maximum execution time in milliseconds
    pub max_execution_time: u64,
    /// Maximum object count
    pub max_objects: usize,
    /// Maximum evaluation depth
    pub max_eval_depth: usize,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            max_memory: 100 * 1024 * 1024, // 100MB
            max_execution_time: 30 * 1000,  // 30 seconds
            max_objects: 100_000,
            max_eval_depth: 1000,
        }
    }
}

/// Security context for evaluating Echo code
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Security policy
    pub policy: SecurityPolicy,
    /// Current player/user
    pub current_player: Option<String>,
}

impl SecurityContext {
    /// Create a new security context
    pub fn new(policy: SecurityPolicy) -> Self {
        Self {
            policy,
            current_player: None,
        }
    }

    /// Create a default security context
    pub fn default() -> Self {
        Self::new(SecurityPolicy::default())
    }

    /// Set the current player
    pub fn set_player(&mut self, player: String) {
        self.current_player = Some(player);
    }

    /// Check if a resource usage is within limits
    pub fn check_resource_usage(&self, objects: usize, eval_depth: usize) -> Result<(), SecurityError> {
        if objects > self.policy.max_objects {
            return Err(SecurityError::ResourceLimit {
                resource: "objects".to_string(),
                limit: self.policy.max_objects,
                actual: objects,
            });
        }

        if eval_depth > self.policy.max_eval_depth {
            return Err(SecurityError::ResourceLimit {
                resource: "eval_depth".to_string(),
                limit: self.policy.max_eval_depth,
                actual: eval_depth,
            });
        }

        Ok(())
    }
}

/// Security-related errors
#[derive(thiserror::Error, Debug)]
pub enum SecurityError {
    /// Resource limit exceeded
    #[error("Resource limit exceeded: {resource} (limit: {limit}, actual: {actual})")]
    ResourceLimit {
        resource: String,
        limit: usize,
        actual: usize,
    },
    
    /// Access denied
    #[error("Access denied: {reason}")]
    AccessDenied { reason: String },
    
    /// Security policy violation
    #[error("Security policy violation: {violation}")]
    PolicyViolation { violation: String },
}