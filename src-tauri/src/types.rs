use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    Working,
    Running,
    Compacting,
    Waiting,
    Inactive,
    Stopped,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub state: AgentState,
    pub task: String,
    pub elapsed: u64,
    pub tokens: u64,
    pub started_at: u64,
    pub alarm_stopped: bool,
    pub alarm_error: bool,
    pub alarm_inactive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertInfo {
    pub id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub state: AgentState,
    pub timestamp: u64, // milliseconds
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEvent {
    pub id: i64,
    pub session_id: String,
    pub event_type: String,
    pub tool_name: Option<String>,
    pub description: Option<String>,
    pub timestamp: u64, // unix milliseconds
}

#[derive(Debug, Deserialize)]
pub struct HookPayload {
    pub session_id: String,
    #[serde(default)]
    pub transcript_path: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    pub hook_event_name: String,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub tool_input: Option<serde_json::Value>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub stop_hook_active: Option<bool>,
}
