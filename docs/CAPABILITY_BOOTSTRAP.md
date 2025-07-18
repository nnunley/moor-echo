# ECHO Capability Bootstrap Design

## Overview

The ECHO capability system provides fine-grained security and access control. This document describes how capabilities are bootstrapped when the system starts and how they propagate through the object hierarchy.

## Core Capability Concepts

### 1. Capability Definition

Capabilities are defined at the global level using the `define` keyword:

```echo
define capability ReadProperty(object, property_name);
define capability WriteProperty(object, property_name);
define capability ExecuteVerb(object, verb_name);
define capability CreateObject(parent);
define capability GrantCapability(capability, target);
```

### 2. Capability Requirements

Functions and verbs can require capabilities:

```echo
secure function modify_property(obj, prop, value)
    requires WriteProperty(obj, prop)
    obj.(prop) = value;
endfunction

secure verb "set" (this, prepstr, value)
    requires WriteProperty(this, "value")
    this.value = value;
endverb
```

### 3. Capability Granting

Capabilities are granted using the `grant` statement:

```echo
grant WriteProperty(my_object, "name") to player;
grant ExecuteVerb(my_object, "look") to $all_players;
```

## Bootstrap Process

### Phase 1: System Object Creation

1. **System Object (#0)** is created with all capabilities
2. System object has implicit `GrantCapability` for all capabilities
3. System object creates core singleton objects

### Phase 2: Core Capability Distribution

```echo
// Bootstrap capabilities for core objects
object $system
    function bootstrap_capabilities()
        // Grant root_class basic capabilities
        grant CreateObject($root_class) to $root_class;
        grant ReadProperty($root_class, any) to $all;
        
        // Grant player creation capabilities
        grant CreateObject($player) to $player_creator;
        grant GrantCapability(ReadProperty, any) to $player_creator;
        
        // Grant room capabilities
        grant ExecuteVerb($room, "look") to $all;
        grant ReadProperty($room, "name") to $all;
        grant ReadProperty($room, "description") to $all;
    endfunction
endobject
```

### Phase 3: Player Capabilities

When a new player is created:

```echo
object $player_creator
    function create_player(name)
        let new_player = create_object($player);
        
        // Grant basic player capabilities
        grant ReadProperty(new_player, any) to new_player;
        grant WriteProperty(new_player, "password") to new_player;
        grant ExecuteVerb(new_player, "look") to new_player;
        grant ExecuteVerb(new_player, "say") to new_player;
        
        // Grant interaction capabilities
        grant ReadProperty(any, "name") to new_player;
        grant ReadProperty(any, "description") to new_player;
        
        return new_player;
    endfunction
endobject
```

### Phase 4: Object Creation Capabilities

When objects are created, they inherit capabilities from their parent:

```echo
function create_object_with_capabilities(parent)
    let obj = create_object(parent);
    
    // Copy parent's grantable capabilities
    for cap in parent.$grantable_capabilities
        grant cap to obj;
    endfor
    
    return obj;
endfunction
```

## Capability Hierarchy

### 1. Universal Capabilities
- `ReadProperty(object, "name")` - Everyone can read names
- `ExecuteVerb(object, "look")` - Everyone can look at objects

### 2. Owner Capabilities
- `WriteProperty(owned_object, any)` - Owners can modify their objects
- `GrantCapability(any, owned_object)` - Owners can grant capabilities on their objects

### 3. Wizard Capabilities
- `CreateObject(any)` - Wizards can create any object type
- `GrantCapability(any, any)` - Wizards can grant any capability
- `ExecuteVerb(any, any)` - Wizards can execute any verb

### 4. System Capabilities
- `ModifySystemObject()` - Only system can modify system objects
- `ShutdownSystem()` - Only system can shut down
- `BootstrapCapabilities()` - Only system can bootstrap

## Implementation in Echo REPL

### Storage Layer

Add capability storage to objects:

```rust
pub struct EchoObject {
    // ... existing fields ...
    pub granted_capabilities: HashMap<String, Vec<CapabilityGrant>>,
    pub required_capabilities: HashMap<String, Vec<CapabilityRequirement>>,
}

pub struct CapabilityGrant {
    pub capability: String,
    pub parameters: Vec<String>,
    pub grantee: ObjectId,
    pub grantor: ObjectId,
    pub timestamp: u64,
}
```

### Evaluator Integration

Check capabilities before operations:

```rust
impl Evaluator {
    fn check_capability(&self, actor: ObjectId, capability: &str, params: &[Value]) -> Result<bool> {
        // Check if actor has the required capability
        // Walk up the inheritance chain if needed
        // Check wildcard grants
    }
    
    fn execute_verb(&mut self, obj: ObjectId, verb: &str, args: &[Value]) -> Result<Value> {
        // Check verb's required capabilities
        let verb_def = self.get_verb(obj, verb)?;
        for required_cap in &verb_def.required_capabilities {
            if !self.check_capability(self.current_player, required_cap, args)? {
                return Err(anyhow!("Permission denied: missing capability {}", required_cap));
            }
        }
        
        // Execute verb
        self.eval_verb_body(&verb_def.body, args)
    }
}
```

## Security Considerations

1. **Capability Leasing**: Capabilities can have expiration times
2. **Delegation Chains**: Track who granted what to prevent cycles
3. **Revocation**: Support revoking previously granted capabilities
4. **Audit Trail**: Log all capability grants and checks

## Example Usage

```echo
// Define a secure container
object secure_vault extends $thing
    property contents = [];
    property access_list = [];
    
    secure verb "open" (this, none, none)
        requires AccessVault(this)
        caller:tell("The vault opens.");
        this.is_open = true;
    endverb
    
    secure function add_access(player)
        requires GrantCapability(AccessVault(this), player)
        grant AccessVault(this) to player;
        this.access_list = {@this.access_list, player};
    endfunction
endobject

// Usage
let vault = create(secure_vault);
vault:add_access(player);  // Requires appropriate capability
player:open(vault);        // Now player can open the vault
```