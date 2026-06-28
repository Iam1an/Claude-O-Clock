use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::sound::SoundKind;
use crate::types::{AgentInfo, AgentState, AlertInfo, HookPayload};

pub type SharedStore = Arc<Mutex<AgentStore>>;

const MAX_AGENTS: usize = 50;

pub struct AgentStore {
    agents: HashMap<String, AgentInfo>,
    dnd: bool,
}

impl AgentStore {
    pub fn new() -> Self {
        Self { agents: HashMap::new(), dnd: false }
    }

    pub fn set_dnd(&mut self, enabled: bool) {
        self.dnd = enabled;
    }

    /// Load a persisted agent from the database on startup.
    pub fn load_agent(&mut self, agent: AgentInfo) {
        self.agents.insert(agent.id.clone(), agent);
    }

    pub fn list(&self) -> Vec<AgentInfo> {
        let mut v: Vec<AgentInfo> = self.agents.values().cloned().collect();
        v.sort_by_key(|a| a.started_at);
        v
    }

    /// Process a hook payload.
    /// Returns the sound to play (if any) and an alert to persist/emit (if any).
    /// Alerts are generated regardless of DND; sounds are suppressed by DND.
    pub fn apply_hook(&mut self, payload: HookPayload) -> (Option<SoundKind>, Option<AlertInfo>) {
        // Reject malformed payloads before touching internal state
        if payload.session_id.is_empty() || payload.session_id.len() > 128 {
            return (None, None);
        }

        let now = unix_now();
        let new_state = infer_state(&payload);
        let task = infer_task(&payload);

        // Scope the mutable borrow of self.agents so we can prune and read below
        let (old_state, current_state, alarm_done, alarm_error, alarm_waiting, agent_name) = {
            let agent = self
                .agents
                .entry(payload.session_id.clone())
                .or_insert_with(|| AgentInfo {
                    id: payload.session_id.clone(),
                    name: derive_name(payload.transcript_path.as_deref(), &payload.session_id),
                    state: AgentState::Thinking,
                    task: String::new(),
                    elapsed: 0,
                    tokens: 0,
                    started_at: now,
                    alarm_done: true,
                    alarm_error: true,
                    alarm_waiting: true,
                });

            let old = agent.state.clone();
            agent.state = new_state;
            agent.elapsed = now.saturating_sub(agent.started_at);
            if !task.is_empty() {
                agent.task = task;
            }
            (
                old,
                agent.state.clone(),
                agent.alarm_done,
                agent.alarm_error,
                agent.alarm_waiting,
                agent.name.clone(),
            )
            // agent borrow dropped here — safe to use self.agents again
        };

        // Prune oldest Done agents when the cap is exceeded
        if self.agents.len() > MAX_AGENTS {
            let mut done_ids: Vec<(u64, String)> = self
                .agents
                .values()
                .filter(|a| a.state == AgentState::Done)
                .map(|a| (a.started_at, a.id.clone()))
                .collect();
            done_ids.sort_unstable_by_key(|&(t, _)| t);
            let excess = self.agents.len().saturating_sub(MAX_AGENTS);
            for (_, id) in done_ids.iter().take(excess) {
                self.agents.remove(id);
            }
        }

        // Sound — suppressed by DND
        let sound = if self.dnd {
            None
        } else {
            sound_for_transition(&old_state, &current_state, alarm_done, alarm_error, alarm_waiting)
        };

        // Alert — generated regardless of DND (always logged in Alerts tab)
        let alert = make_alert(
            &payload.session_id,
            &agent_name,
            &old_state,
            &current_state,
            alarm_done,
            alarm_error,
            alarm_waiting,
        );

        (sound, alert)
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn unix_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn infer_state(p: &HookPayload) -> AgentState {
    match p.hook_event_name.as_str() {
        "PreToolUse" => AgentState::Doing,
        "PostToolUse" => AgentState::Thinking,
        "Stop" => AgentState::Done,
        "Notification" => {
            let msg = p.message.as_deref().unwrap_or("").to_ascii_lowercase();
            // Compaction is the only notification reliable enough to change state.
            // "Waiting" is NOT detected here — the word appears in many unrelated
            // messages and causes false alarms (e.g. "please wait…" status updates).
            if msg.contains("compact") {
                AgentState::Compacting
            } else {
                AgentState::Thinking
            }
        }
        _ => AgentState::Thinking,
    }
}

fn infer_task(p: &HookPayload) -> String {
    match p.hook_event_name.as_str() {
        "PreToolUse" => {
            let tool = p.tool_name.as_deref().unwrap_or("tool");
            if let Some(input) = &p.tool_input {
                if let Some(cmd) = input.get("command").and_then(|v| v.as_str()) {
                    let s = cmd.trim();
                    return format!("{tool}: {}", &s[..s.len().min(60)]);
                }
                for key in &["path", "file_path", "url"] {
                    if let Some(val) = input.get(key).and_then(|v| v.as_str()) {
                        return format!("{tool}: {}", &val[..val.len().min(60)]);
                    }
                }
            }
            format!("Running {tool}")
        }
        "Stop" => "Task complete".to_string(),
        "Notification" => {
            let msg = p.message.as_deref().unwrap_or("").to_ascii_lowercase();
            if msg.contains("compact") {
                "Compacting context...".to_string()
            } else {
                // Empty — keep the last real task description from PreToolUse.
                // Generic notifications must not overwrite it.
                String::new()
            }
        }
        _ => String::new(),
    }
}

fn sound_for_transition(
    old: &AgentState,
    new: &AgentState,
    alarm_done: bool,
    alarm_error: bool,
    alarm_waiting: bool,
) -> Option<SoundKind> {
    if old == new {
        return None;
    }
    match new {
        AgentState::Done if alarm_done => Some(SoundKind::Done),
        AgentState::Error if alarm_error => Some(SoundKind::Urgent),
        AgentState::Waiting if alarm_waiting => Some(SoundKind::Urgent),
        _ => None,
    }
}

fn make_alert(
    session_id: &str,
    agent_name: &str,
    old: &AgentState,
    new: &AgentState,
    alarm_done: bool,
    alarm_error: bool,
    alarm_waiting: bool,
) -> Option<AlertInfo> {
    if old == new {
        return None;
    }
    let enabled = match new {
        AgentState::Done => alarm_done,
        AgentState::Error => alarm_error,
        AgentState::Waiting => alarm_waiting,
        _ => return None,
    };
    if !enabled {
        return None;
    }
    let ts = unix_now_ms();
    Some(AlertInfo {
        id: format!("{session_id}-{ts}"),
        agent_id: session_id.to_string(),
        agent_name: agent_name.to_string(),
        state: new.clone(),
        timestamp: ts,
        message: format!("{agent_name} → {}", state_label(new)),
    })
}

fn state_label(state: &AgentState) -> &'static str {
    match state {
        AgentState::Thinking => "Thinking",
        AgentState::Planning => "Planning",
        AgentState::Doing => "Doing",
        AgentState::Compacting => "Compacting",
        AgentState::Done => "Done",
        AgentState::Error => "Error",
        AgentState::Waiting => "Waiting",
    }
}

// Claude Code encodes the project path by replacing / with -
// e.g., /Users/ian/Desktop/my-project → -Users-ian-Desktop-my-project
fn derive_name(transcript_path: Option<&str>, session_id: &str) -> String {
    if let Some(path) = transcript_path {
        if let Some(idx) = path.find("/projects/") {
            let after = &path[idx + 10..];
            let encoded = match after.find('/') {
                Some(slash) => &after[..slash],
                None => after,
            };
            let name = clean_encoded_path(encoded);
            if !name.is_empty() {
                return name;
            }
        }
    }
    format!("agent-{}", &session_id[..session_id.len().min(6)])
}

fn clean_encoded_path(encoded: &str) -> String {
    let stripped = encoded.trim_start_matches('-');

    if let Ok(home) = std::env::var("HOME") {
        let home_enc = home.trim_start_matches('/').replace('/', "-");
        if let Some(after_home) = stripped.strip_prefix(home_enc.as_str()) {
            let rest = after_home.trim_start_matches('-');
            let common = [
                "Desktop", "Documents", "Downloads", "dev", "src",
                "code", "workspace", "repos", "projects", "work",
            ];
            for dir in &common {
                if let Some(proj) = rest.strip_prefix(&format!("{dir}-")) {
                    return proj.to_string();
                }
                if rest.eq_ignore_ascii_case(dir) {
                    return rest.to_string();
                }
            }
            if !rest.is_empty() {
                return rest.to_string();
            }
        }
    }

    let parts: Vec<&str> = stripped.split('-').collect();
    let mut i = if parts
        .first()
        .map_or(false, |p| p.eq_ignore_ascii_case("users") || p.eq_ignore_ascii_case("home"))
    {
        2
    } else {
        0
    };
    let skip = [
        "desktop", "documents", "downloads", "dev", "src",
        "code", "workspace", "repos", "work", "projects",
    ];
    while i < parts.len() && skip.iter().any(|d| d.eq_ignore_ascii_case(parts[i])) {
        i += 1;
    }
    if i < parts.len() {
        parts[i..].join("-")
    } else {
        parts.last().map(|s| s.to_string()).unwrap_or_default()
    }
}
