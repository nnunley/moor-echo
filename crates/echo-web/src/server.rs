//! Web server implementation for Echo
//!
//! Provides HTTP and WebSocket endpoints for interacting with Echo runtime
//! through a web interface.

use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use anyhow::Result;
use axum::{
    extract::{ws::WebSocket, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{Html, Response},
    routing::{get, post},
    Json, Router,
};
use echo_core::{EchoRuntime, Value};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::web_notifier::WebNotifier as WebReplNotifier;

/// Web server configuration
#[derive(Debug, Clone)]
pub struct WebServerConfig {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Static files directory
    pub static_dir: PathBuf,
    /// Enable CORS
    pub enable_cors: bool,
}

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// Echo runtime
    pub runtime: Arc<RwLock<EchoRuntime>>,
    /// Web notifier for REPL integration
    pub notifier: Arc<WebReplNotifier>,
}

/// Request to execute Echo code
#[derive(Deserialize)]
pub struct ExecuteRequest {
    /// Code to execute
    code: String,
    /// Whether this is a program (multi-statement)
    #[serde(default)]
    is_program: bool,
}

/// Request for command API (compatible with original implementation)
#[derive(Deserialize)]
pub struct CommandRequest {
    /// Command to execute
    command: String,
}

/// Response from code execution
#[derive(Serialize)]
pub struct ExecuteResponse {
    /// Execution result
    result: String,
    /// Execution time in milliseconds
    duration_ms: u64,
    /// Whether execution was successful
    success: bool,
    /// Error message if unsuccessful
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Response from command execution (compatible with original implementation)
#[derive(Serialize)]
pub struct CommandResponse {
    /// Whether command was successful
    success: bool,
    /// Error message if any
    message: Option<String>,
}

/// Query parameters for state requests
#[derive(Deserialize)]
pub struct StateQuery {
    /// Optional filter for specific state components
    #[allow(dead_code)]
    filter: Option<String>,
}

/// Web server for Echo runtime
pub struct WebServer {
    /// Server configuration
    config: WebServerConfig,
    /// Application state
    state: AppState,
}

impl WebServer {
    /// Create a new web server
    pub fn new(config: WebServerConfig, mut runtime: EchoRuntime) -> Self {
        let notifier = Arc::new(WebReplNotifier::new(100));

        // Set up UI callback to send UI events to WebNotifier
        let notifier_clone = notifier.clone();
        let ui_callback = std::sync::Arc::new(move |ui_event: echo_core::UiEvent| {
            use crate::web_notifier::UiUpdate;

            let update = match ui_event.action {
                echo_core::UiAction::Clear => UiUpdate {
                    target: "dynamicContent".to_string(),
                    action: "clear".to_string(),
                    data: serde_json::json!({}),
                },
                echo_core::UiAction::AddButton { id, label, action } => UiUpdate {
                    target: id,
                    action: "add_button".to_string(),
                    data: serde_json::json!({
                        "label": label,
                        "action": action
                    }),
                },
                echo_core::UiAction::AddText { id, text, style } => UiUpdate {
                    target: id,
                    action: "add_text".to_string(),
                    data: serde_json::json!({
                        "text": text,
                        "style": style
                    }),
                },
                echo_core::UiAction::AddDiv { id, content, style } => UiUpdate {
                    target: id,
                    action: "add_div".to_string(),
                    data: serde_json::json!({
                        "content": content,
                        "style": style
                    }),
                },
                echo_core::UiAction::Update { id, properties } => UiUpdate {
                    target: id,
                    action: "update".to_string(),
                    data: serde_json::json!(properties),
                },
                echo_core::UiAction::NotifyPlayer { player_id, message } => {
                    // Handle player notifications directly through the notifier
                    let player_str = format!("{}", player_id);
                    notifier_clone.send_player_notification(&player_str, &message);
                    // Return a dummy update since we handled it directly
                    UiUpdate {
                        target: format!("player_{}", player_id),
                        action: "notify".to_string(),
                        data: serde_json::json!({
                            "message": message
                        }),
                    }
                },
            };

            notifier_clone.send_ui_update(update);
        });

        runtime.set_ui_callback(ui_callback);

        let state = AppState {
            runtime: Arc::new(RwLock::new(runtime)),
            notifier,
        };

        Self { config, state }
    }

    /// Get the web notifier for REPL integration
    pub fn notifier(&self) -> Arc<WebReplNotifier> {
        self.state.notifier.clone()
    }

    /// Start the web server
    pub async fn start(self) -> Result<()> {
        let config = self.config.clone();
        let app = self.create_router().await?;

        let addr = format!("{}:{}", config.host, config.port).parse::<SocketAddr>()?;

        println!("Web server starting on http://{addr}");

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    /// Create the router with all routes
    async fn create_router(self) -> Result<Router> {
        let mut router = Router::new()
            .route("/ws", get(websocket_handler))
            .route("/api/execute", post(execute_handler))
            .route("/api/command", post(command_handler))
            .route("/api/state", get(state_handler))
            .route("/", get(index_handler))
            .with_state(self.state);

        // Add static file serving
        if self.config.static_dir.exists() {
            router = router.nest_service("/static", ServeDir::new(&self.config.static_dir));
        }

        // Add CORS if enabled
        if self.config.enable_cors {
            router = router.layer(ServiceBuilder::new().layer(CorsLayer::permissive()));
        }

        Ok(router)
    }
}

/// WebSocket connection handler
async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

/// Execute a request and return the response
async fn execute_request(request: &ExecuteRequest, state: &AppState) -> ExecuteResponse {
    let mut runtime = state.runtime.write().await;
    let start = std::time::Instant::now();

    let execution_result = if request.is_program {
        runtime
            .parse_program(&request.code)
            .and_then(|ast| runtime.eval(&ast))
    } else {
        runtime.eval_source(&request.code)
    };

    let duration = start.elapsed().as_millis() as u64;

    match execution_result {
        Ok(value) => {
            let result = format_value(&value);
            // Send result event to all connected clients
            state
                .notifier
                .send_result(&result, std::time::Duration::from_millis(duration));
            ExecuteResponse {
                result,
                duration_ms: duration,
                success: true,
                error: None,
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            // Send error event to all connected clients
            state.notifier.send_error(&error_msg);
            ExecuteResponse {
                result: String::new(),
                duration_ms: duration,
                success: false,
                error: Some(error_msg),
            }
        }
    }
}

/// Handle WebSocket connections
async fn handle_websocket(socket: WebSocket, state: AppState) {
    let mut rx = state.notifier.subscribe();
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(tokio::sync::Mutex::new(sender));

    // Spawn task to handle incoming messages
    let state_clone = state.clone();
    let sender_clone = sender.clone();
    let incoming_task = tokio::spawn(async move {
        use axum::extract::ws::Message;

        while let Some(msg) = receiver.next().await {
            if let Ok(Message::Text(text)) = msg {
                // Handle incoming WebSocket messages (commands from UI)
                if let Ok(request) = serde_json::from_str::<ExecuteRequest>(&text) {
                    let result = execute_request(&request, &state_clone).await;

                    let response = serde_json::to_string(&result).unwrap_or_default();
                    let mut sender = sender_clone.lock().await;
                    let _ = sender.send(Message::Text(response)).await;
                }
            }
        }
    });

    // Handle outgoing messages (events to UI)
    let outgoing_task = tokio::spawn(async move {
        use axum::extract::ws::Message;

        while let Ok(event) = rx.recv().await {
            let json = serde_json::to_string(&event).unwrap_or_default();
            let mut sender = sender.lock().await;
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = incoming_task => {},
        _ = outgoing_task => {},
    }
}

/// Execute Echo code via HTTP API
async fn execute_handler(
    State(state): State<AppState>,
    Json(request): Json<ExecuteRequest>,
) -> Result<Json<ExecuteResponse>, StatusCode> {
    let mut runtime = state.runtime.write().await;
    let start = std::time::Instant::now();

    let result = if request.is_program {
        runtime
            .parse_program(&request.code)
            .and_then(|ast| runtime.eval(&ast))
    } else {
        runtime.eval_source(&request.code)
    };

    let duration = start.elapsed().as_millis() as u64;

    let response = match result {
        Ok(value) => {
            let result = format_value(&value);
            // Send result event to all connected clients
            state
                .notifier
                .send_result(&result, std::time::Duration::from_millis(duration));
            ExecuteResponse {
                result,
                duration_ms: duration,
                success: true,
                error: None,
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            // Send error event to all connected clients
            state.notifier.send_error(&error_msg);
            ExecuteResponse {
                result: String::new(),
                duration_ms: duration,
                success: false,
                error: Some(error_msg),
            }
        }
    };

    Ok(Json(response))
}

/// Execute Echo command via HTTP API (compatible with original implementation)
async fn command_handler(
    State(state): State<AppState>,
    Json(request): Json<CommandRequest>,
) -> Result<Json<CommandResponse>, StatusCode> {
    let mut runtime = state.runtime.write().await;

    let response = if request.command.starts_with('.') {
        // Handle REPL commands
        let parts: Vec<&str> = request.command.split_whitespace().collect();
        match parts.as_slice() {
            [".help"] => CommandResponse {
                success: true,
                message: Some(
                    "Available commands:\n.help - Show this help\n.env - Show environment \
                     variables\n.player list - List players\n.player create <name> - Create \
                     player\n.player switch <name> - Switch player\n.player - Show current \
                     player\n.say <message> - Send chat message to all players\n.quit - Not \
                     applicable in web mode"
                        .to_string(),
                ),
            },
            [".env"] => {
                // Show environment variables - would need access to evaluator's environment
                CommandResponse {
                    success: true,
                    message: Some("Environment variables shown in the UI table".to_string()),
                }
            }
            [".player", "list"] => match runtime.list_players() {
                Ok(players) => {
                    let player_list = players
                        .iter()
                        .map(|(name, id)| format!("  {name} ({id})"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    CommandResponse {
                        success: true,
                        message: Some(format!("Players:\n{player_list}")),
                    }
                }
                Err(e) => CommandResponse {
                    success: false,
                    message: Some(format!("Error listing players: {e}")),
                },
            },
            [".player", "create", name] => match runtime.create_player(name) {
                Ok(player_id) => {
                    let _ = runtime.switch_player(player_id);
                    CommandResponse {
                        success: true,
                        message: Some(format!(
                            "Created and switched to player '{name}' ({player_id})"
                        )),
                    }
                }
                Err(e) => CommandResponse {
                    success: false,
                    message: Some(format!("Error creating player: {e}")),
                },
            },
            [".player", "switch", name] => match runtime.switch_player_by_name(name) {
                Ok(_) => CommandResponse {
                    success: true,
                    message: Some(format!("Switched to player '{name}'")),
                },
                Err(e) => CommandResponse {
                    success: false,
                    message: Some(format!("Error switching player: {e}")),
                },
            },
            [".player"] => {
                match runtime.current_player() {
                    Some(player_id) => {
                        // Try to get player name
                        let player_info = match runtime.list_players() {
                            Ok(players) => players
                                .iter()
                                .find(|(_, id)| *id == player_id)
                                .map(|(name, _)| format!("Current player: {name} ({player_id})"))
                                .unwrap_or_else(|| format!("Current player: {player_id}")),
                            Err(_) => format!("Current player: {player_id}"),
                        };
                        CommandResponse {
                            success: true,
                            message: Some(player_info),
                        }
                    }
                    None => CommandResponse {
                        success: true,
                        message: Some("No player selected".to_string()),
                    },
                }
            }
            [".say", rest @ ..] => {
                if rest.is_empty() {
                    CommandResponse {
                        success: false,
                        message: Some("Usage: .say <message>".to_string()),
                    }
                } else {
                    let message = rest.join(" ");
                    // Get current player name for the message
                    let player_name = match runtime.current_player() {
                        Some(player_id) => match runtime.list_players() {
                            Ok(players) => players
                                .iter()
                                .find(|(_, id)| *id == player_id)
                                .map(|(name, _)| name.clone())
                                .unwrap_or_else(|| player_id.to_string()),
                            Err(_) => player_id.to_string(),
                        },
                        None => "anonymous".to_string(),
                    };

                    // Broadcast chat message to all connected clients
                    state.notifier.send_chat_message(&player_name, &message);

                    CommandResponse {
                        success: true,
                        message: Some(format!("You say: {message}")),
                    }
                }
            }
            [".quit"] => CommandResponse {
                success: true,
                message: Some("Quit not applicable in web mode".to_string()),
            },
            _ => CommandResponse {
                success: false,
                message: Some(format!(
                    "Unknown command: {}. Type .help for available commands.",
                    request.command
                )),
            },
        }
    } else {
        // Echo code
        match runtime.eval_source(&request.command) {
            Ok(_) => {
                // Send state update after successful execution
                let env_vars = runtime.get_environment_vars();
                let environment: Vec<crate::web_notifier::EnvironmentVar> = env_vars
                    .into_iter()
                    .map(|(name, value)| crate::web_notifier::EnvironmentVar {
                        name,
                        value: format_value(&value),
                        var_type: value.type_name().to_string(),
                    })
                    .collect();

                // Get current player name
                let current_player = match runtime.current_player() {
                    Some(player_id) => {
                        // Try to get player name
                        match runtime.list_players() {
                            Ok(players) => players
                                .iter()
                                .find(|(_, id)| *id == player_id)
                                .map(|(name, _)| name.clone())
                                .unwrap_or_else(|| player_id.to_string()),
                            Err(_) => player_id.to_string(),
                        }
                    }
                    None => "default".to_string(),
                };

                let snapshot = crate::web_notifier::StateSnapshot {
                    environment,
                    objects: Vec::new(), // TODO: Implement object listing
                    current_player,
                };

                state.notifier.send_state_update(snapshot);

                CommandResponse {
                    success: true,
                    message: None,
                }
            }
            Err(e) => CommandResponse {
                success: false,
                message: Some(format!("Execution error: {e}")),
            },
        }
    };

    Ok(Json(response))
}

/// Get runtime state via HTTP API
async fn state_handler(
    State(_state): State<AppState>,
    Query(_params): Query<StateQuery>,
) -> Json<serde_json::Value> {
    // Return basic state information
    let state_info = serde_json::json!({
        "status": "running",
        "version": crate::VERSION,
        "features": echo_core::features()
    });

    Json(state_info)
}

/// Serve the main index page
async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

/// Format a value for web display
fn format_value(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => format!("\"{s}\""),
        Value::List(items) => {
            let formatted: Vec<String> = items.iter().map(format_value).collect();
            format!("[{}]", formatted.join(", "))
        }
        Value::Object(id) => format!("#{id}"),
        Value::Map(map) => {
            let formatted: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_value(v)))
                .collect();
            format!("{{{}}}", formatted.join(", "))
        }
        Value::Lambda { .. } => "<lambda>".to_string(),
    }
}
