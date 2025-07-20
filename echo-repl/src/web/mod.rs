use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use axum::{
    Router,
    routing::{get, post},
    extract::{ws::{WebSocket, WebSocketUpgrade}, State, Json},
    response::{IntoResponse, Html},
};
use serde::{Serialize, Deserialize};
use futures::{sink::SinkExt, stream::StreamExt};
use crate::repl::Repl;
use super::repl::web_notifier::{WebNotifier, WebClient, StateSnapshot, EnvironmentVar, ObjectInfo};

pub struct WebServer {
    repl: Arc<TokioMutex<Repl>>,
    notifier: Arc<WebNotifier>,
}

#[derive(Clone)]
struct AppState {
    repl: Arc<TokioMutex<Repl>>,
    notifier: Arc<WebNotifier>,
}

#[derive(Deserialize)]
struct CommandRequest {
    command: String,
}

#[derive(Serialize)]
struct CommandResponse {
    success: bool,
    message: Option<String>,
}

impl WebServer {
    pub fn new(repl: Arc<TokioMutex<Repl>>, notifier: Arc<WebNotifier>) -> Self {
        Self { repl, notifier }
    }

    pub fn routes(&self) -> Router {
        let state = AppState {
            repl: self.repl.clone(),
            notifier: self.notifier.clone(),
        };

        Router::new()
            .route("/", get(serve_ui))
            .route("/ws", get(websocket_handler))
            .route("/api/command", post(execute_command))
            .route("/api/state", get(get_state))
            .route("/api/environment", get(get_environment))
            .route("/api/objects", get(get_objects))
            .with_state(state)
    }
}

async fn serve_ui() -> impl IntoResponse {
    Html(include_str!("../../static/index.html"))
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (_client, mut event_receiver) = WebClient::new(state.notifier.clone());

    // Task to forward events from notifier to WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            let msg = match serde_json::to_string(&event) {
                Ok(json) => axum::extract::ws::Message::Text(json),
                Err(_) => continue,
            };
            
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Task to handle incoming WebSocket messages
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                axum::extract::ws::Message::Text(_text) => {
                    // Handle incoming commands if needed
                }
                axum::extract::ws::Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

async fn execute_command(
    State(state): State<AppState>,
    Json(request): Json<CommandRequest>,
) -> impl IntoResponse {
    let mut repl = state.repl.lock().await;
    
    let response = if request.command.starts_with('.') {
        // REPL command
        match repl.parse_input(&request.command) {
            Ok(command) => {
                match repl.handle_command(command) {
                    Ok(_) => CommandResponse {
                        success: true,
                        message: None,
                    },
                    Err(e) => CommandResponse {
                        success: false,
                        message: Some(format!("Error: {}", e)),
                    },
                }
            }
            Err(e) => CommandResponse {
                success: false,
                message: Some(format!("Parse error: {}", e)),
            },
        }
    } else {
        // Echo code
        match repl.execute(&request.command) {
            Ok(_) => CommandResponse {
                success: true,
                message: None,
            },
            Err(e) => CommandResponse {
                success: false,
                message: Some(format!("Execution error: {}", e)),
            },
        }
    };

    Json(response)
}

async fn get_state(State(state): State<AppState>) -> impl IntoResponse {
    let repl = state.repl.lock().await;
    
    // Create state snapshot
    let snapshot = create_state_snapshot(&*repl);
    
    Json(snapshot)
}

async fn get_environment(State(state): State<AppState>) -> impl IntoResponse {
    let repl = state.repl.lock().await;
    let env_vars = get_environment_vars(&*repl);
    
    Json(env_vars)
}

async fn get_objects(State(state): State<AppState>) -> impl IntoResponse {
    let repl = state.repl.lock().await;
    let objects = get_object_list(&*repl);
    
    Json(objects)
}

fn create_state_snapshot(repl: &Repl) -> StateSnapshot {
    StateSnapshot {
        environment: get_environment_vars(repl),
        objects: get_object_list(repl),
        current_player: repl.current_player_name().unwrap_or_default(),
    }
}

fn get_environment_vars(repl: &Repl) -> Vec<EnvironmentVar> {
    repl.get_environment_snapshot()
        .into_iter()
        .map(|(name, value, var_type)| EnvironmentVar {
            name,
            value,
            var_type,
        })
        .collect()
}

fn get_object_list(repl: &Repl) -> Vec<ObjectInfo> {
    repl.get_objects_snapshot()
        .into_iter()
        .map(|(id, name, properties)| ObjectInfo {
            id,
            name,
            properties,
        })
        .collect()
}