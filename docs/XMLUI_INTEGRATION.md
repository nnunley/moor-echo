# XML UI Integration for Echo REPL

This document outlines the design for integrating XML UI
(https://blog.jonudell.net/2025/07/18/introducing-xmlui/) with the Echo REPL to
provide a web-based interface that can be influenced from within the REPL
itself.

## Overview

The integration will create a web-based UI for the Echo REPL that:

1. Displays REPL output in real-time
2. Allows sending commands to the REPL
3. Shows current REPL state (environment, objects, etc.)
4. Can be dynamically modified from within the REPL itself

## Architecture

### 1. WebNotifier Implementation

Create a custom `ReplNotifier` that streams output to web clients:

```rust
// src/repl/web_notifier.rs
pub struct WebNotifier {
    sender: Arc<Mutex<Option<UnboundedSender<WebEvent>>>>,
    buffer: Arc<Mutex<VecDeque<WebEvent>>>,
    buffer_size: usize,
}

pub enum WebEvent {
    Output(String),
    Error(String),
    Result { output: String, duration: Duration },
    StateUpdate(StateSnapshot),
}

impl ReplNotifier for WebNotifier {
    fn on_output(&self, output: &str) {
        self.send_event(WebEvent::Output(output.to_string()));
    }

    fn on_error(&self, error: &str) {
        self.send_event(WebEvent::Error(error.to_string()));
    }

    fn on_result(&self, output: &str, duration: Duration, quiet: bool) {
        if !quiet {
            self.send_event(WebEvent::Result {
                output: output.to_string(),
                duration,
            });
        }
    }
}
```

### 2. Web Server Module

Add a web server with WebSocket support:

```rust
// src/web/mod.rs
use axum::{
    Router,
    routing::{get, post},
    extract::ws::{WebSocket, WebSocketUpgrade},
};

pub struct WebServer {
    repl: Arc<Mutex<Repl>>,
    notifier: Arc<WebNotifier>,
}

impl WebServer {
    pub fn routes() -> Router {
        Router::new()
            .route("/", get(serve_ui))
            .route("/ws", get(websocket_handler))
            .route("/api/command", post(execute_command))
            .route("/api/state", get(get_state))
            .route("/api/environment", get(get_environment))
            .route("/api/objects", get(get_objects))
    }
}
```

### 3. XML UI Components

Design XML UI components for the REPL interface:

```xml
<!-- index.html -->
<xmlui>
  <App title="Echo REPL">
    <VStack>
      <!-- Output Display -->
      <Card title="REPL Output" height="400px">
        <DataSource
          id="replOutput"
          url="/api/output"
          websocket="/ws"
          onMessage="handleWebSocketMessage">
        </DataSource>
        <ScrollView>
          <VStack id="outputContainer">
            <!-- Output lines will be dynamically added here -->
          </VStack>
        </ScrollView>
      </Card>

      <!-- Command Input -->
      <Card title="Command Input">
        <HStack>
          <TextBox
            id="commandInput"
            placeholder="Enter command..."
            onEnter="sendCommand">
          </TextBox>
          <Button onClick="sendCommand">Send</Button>
        </HStack>
      </Card>

      <!-- State Display -->
      <HStack>
        <!-- Environment Variables -->
        <Card title="Environment" flex="1">
          <DataSource
            id="environment"
            url="/api/environment"
            refresh="onStateChange">
          </DataSource>
          <Table data="{environment}" height="200px">
            <Column bindTo="name" title="Variable"/>
            <Column bindTo="value" title="Value"/>
            <Column bindTo="type" title="Type"/>
          </Table>
        </Card>

        <!-- Objects -->
        <Card title="Objects" flex="1">
          <DataSource
            id="objects"
            url="/api/objects"
            refresh="onStateChange">
          </DataSource>
          <Table data="{objects}" height="200px">
            <Column bindTo="id" title="ID"/>
            <Column bindTo="name" title="Name"/>
            <Column bindTo="properties" title="Properties"/>
          </Table>
        </Card>
      </HStack>

      <!-- Dynamic UI Area -->
      <Card title="Dynamic UI" id="dynamicArea">
        <!-- This area can be modified from within the REPL -->
      </Card>
    </VStack>
  </App>

  <Script>
    function handleWebSocketMessage(event) {
      const data = JSON.parse(event.data);
      const outputContainer = document.getElementById('outputContainer');

      switch(data.type) {
        case 'output':
          addOutputLine(data.content, 'output');
          break;
        case 'error':
          addOutputLine(data.content, 'error');
          break;
        case 'result':
          addOutputLine(data.output, 'result');
          break;
        case 'stateUpdate':
          // Refresh data sources
          xmlui.refresh('environment');
          xmlui.refresh('objects');
          break;
        case 'uiUpdate':
          // Handle dynamic UI updates from REPL
          updateDynamicUI(data.update);
          break;
      }
    }

    function sendCommand() {
      const input = document.getElementById('commandInput');
      const command = input.value;

      fetch('/api/command', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command })
      });

      input.value = '';
    }

    function updateDynamicUI(update) {
      const dynamicArea = document.getElementById('dynamicArea');
      // Apply XML UI updates sent from the REPL
      xmlui.applyUpdate(dynamicArea, update);
    }
  </Script>
</xmlui>
```

### 4. REPL Integration

Add commands and functions to influence the UI from within the REPL:

```rust
// New REPL commands
enum ReplCommand {
    // ... existing commands ...
    UiUpdate(String),  // Update dynamic UI area
    UiClear,          // Clear dynamic UI area
    UiShow(String),   // Show specific UI component
}

// In the evaluator, add built-in functions:
// ui_add_button(id, label, action)
// ui_add_chart(id, data, type)
// ui_add_form(id, fields)
// ui_bind(id, object_property)
```

### 5. Example Usage

From within the REPL, users could:

```echo
# Create a button in the dynamic UI
ui_add_button("myButton", "Click Me", lambda() {
    print("Button clicked!");
    counter = counter + 1;
})

# Add a chart showing object properties
ui_add_chart("propChart", $object.properties, "bar")

# Bind UI element to object property
ui_bind("statusLabel", $system.status)

# Create a form for object creation
ui_add_form("objectCreator", {
    name: "text",
    type: "select:player|room|thing",
    description: "textarea"
})
```

## Implementation Plan

1. **Phase 1**: WebNotifier and basic web server
   - Implement WebNotifier
   - Add axum web server with WebSocket support
   - Basic static UI serving

2. **Phase 2**: XML UI Components
   - Create base XML UI components
   - Implement WebSocket message handling
   - Add output display and command input

3. **Phase 3**: State Display
   - Add REST endpoints for state queries
   - Implement environment and object tables
   - Add auto-refresh on state changes

4. **Phase 4**: Dynamic UI from REPL
   - Add ui\_\* built-in functions
   - Implement UI update protocol
   - Create examples of dynamic UI manipulation

5. **Phase 5**: Advanced Features
   - Add event visualization
   - Implement object property binding
   - Add collaborative features (multiple users)

## Benefits

1. **Visual Feedback**: See REPL output and state in a structured UI
2. **Dynamic Interaction**: Create UI elements from within the REPL
3. **State Monitoring**: Real-time view of environment and objects
4. **Educational**: Visual representation helps understand REPL behavior
5. **Extensible**: XML UI's declarative nature makes it easy to extend

## Technical Considerations

1. **Optional Feature**: Web UI should be an optional feature flag
2. **Security**: Add authentication for web access
3. **Performance**: Use efficient WebSocket protocols for updates
4. **Compatibility**: Ensure REPL works without web UI enabled
5. **State Sync**: Handle concurrent access properly
