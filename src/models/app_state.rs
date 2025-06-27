use super::{config::Config, status::Status};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{
    Mutex,
    broadcast::{self},
};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub builds: BuildState,
    pub project_sender: broadcast::Sender<ChannelMessage>,
    pub build_sender: broadcast::Sender<ChannelMessage>,
    pub is_queue_running: Arc<Mutex<bool>>,
    pub is_terminated: Arc<Mutex<bool>>,
    pub project_token: Arc< Mutex< Option<String> > >,
    pub project_logs:  Arc< Mutex< Vec<ProjectLog> > >,
}

#[derive(Clone,Serialize)]
pub struct ProjectLog {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub unique_id: String,
    pub socket_token: String,
    pub step: usize,
    pub state: Status,
    pub message: String,
}

#[derive(Clone)]
pub struct BuildState {
    pub build_queue: Arc<Mutex<Vec<BuildRequest>>>,
    pub current_build: Arc<Mutex<Option<BuildProcess>>>,
    pub failed_history: Arc<Mutex<Vec<BuildProcess>>>,
}

impl  BuildState {
    pub fn new() -> Self {
        Self {
            build_queue: Arc::new(Mutex::new(Vec::new())),
            current_build: Arc::new(Mutex::new(None)),
            failed_history: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[derive(Clone)]
pub struct BuildRequest {
    pub id: String,
    pub unique_id: String,
    pub payload: HashMap<String, String>,
    pub socket_token: String,
}

// #[derive()]
#[derive(Clone,Serialize)]

pub struct BuildProcess {
    pub id: String,
    pub unique_id: String,
    pub status: Status,
    pub current_step: usize,
    pub total_steps: usize,
    pub started_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub duration: i64,
    pub socket_token: String,
    pub payload: HashMap<String, String>,
    pub out_payload: HashMap<String, String>,
    pub logs: Vec<BuildLog>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildLog {
    pub timestamp: DateTime<Utc>,
    pub status: Status,
    pub step: usize,
    pub message: String,
}

#[derive(Serialize)]
pub struct BuildResponse {
    pub message: String,
    pub status: Status,
    pub token: Option<String>,
    pub build_id: Option<String>,
    // pub payload: Option<serde_json::Value>,
}

#[derive(Clone)]
pub enum ChannelMessage {
    Data(String),
    Shutdown,
}

impl AppState {
    pub async fn new(config: Config) -> Self {
        let (project_sender, _) = broadcast::channel::<ChannelMessage>(100);
        let (build_sender, _) = broadcast::channel::<ChannelMessage>(100);

        Self {
            config,
            is_terminated: Arc::new(Mutex::new(false)),
            project_sender,
            build_sender,
            is_queue_running: Arc::new(Mutex::new(false)),
            builds: BuildState::new(),
            project_token: Arc::new(Mutex::new(None)),
            project_logs: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

