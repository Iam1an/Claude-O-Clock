use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::sound::SoundKind;
use crate::types::{AgentInfo, AgentState, AlertInfo, HookPayload};

pub type SharedStore = Arc<Mutex<AgentStore>>;

const MAX_AGENTS: usize = 50;

pub struct AgentStore {
    agents: HashMap<String, AgentInfo>,
    // Unix-seconds of the last hook received per agent — not serialised.
    last_event_at: HashMap<String, u64>,
    dnd: bool,
}

impl AgentStore {
    pub fn new() -> Self {
        Self { agents: HashMap::new(), last_event_at: HashMap::new(), dnd: false }
    }

    pub fn set_dnd(&mut self, enabled: bool) {
        self.dnd = enabled;
    }

    pub fn load_agent(&mut self, agent: AgentInfo) {
        self.agents.insert(agent.id.clone(), agent);
    }

    pub fn set_agent_alarms(&mut self, id: &str, stopped: bool, error: bool, inactive: bool) {
        if let Some(agent) = self.agents.get_mut(id) {
            agent.alarm_stopped = stopped;
            agent.alarm_error = error;
            agent.alarm_inactive = inactive;
        }
    }

    pub fn rename_agent(&mut self, id: &str, name: &str) {
        if let Some(agent) = self.agents.get_mut(id) {
            agent.name = name.to_string();
        }
    }

    pub fn list(&self) -> Vec<AgentInfo> {
        let mut v: Vec<AgentInfo> = self.agents.values().cloned().collect();
        v.sort_by_key(|a| a.started_at);
        v
    }

    /// Transition agents that have been Working/Running for longer than
    /// `idle_secs` without a new hook into Inactive.
    pub fn tick_idle(&mut self, idle_secs: u64) -> Option<Vec<AgentInfo>> {
        let now = unix_now();
        let mut changed = false;

        for agent in self.agents.values_mut() {
            let active = matches!(agent.state, AgentState::Working | AgentState::Running);
            if !active {
                continue;
            }
            let last = self.last_event_at.get(&agent.id).copied().unwrap_or(agent.started_at);
            if now.saturating_sub(last) >= idle_secs {
                agent.state = AgentState::Inactive;
                changed = true;
            }
        }

        if changed { Some(self.list()) } else { None }
    }

    /// Process a hook payload.
    /// Returns the sound to play (if any) and an alert to persist/emit (if any).
    /// Alerts are always generated; sounds are suppressed by DND.
    pub fn apply_hook(&mut self, payload: HookPayload) -> (Option<SoundKind>, Option<AlertInfo>) {
        if payload.session_id.is_empty() || payload.session_id.len() > 128 {
            return (None, None);
        }

        let now = unix_now();
        self.last_event_at.insert(payload.session_id.clone(), now);

        let new_state = infer_state(&payload);
        let task = infer_task(&payload);

        let (old_state, current_state, alarm_stopped, alarm_error, alarm_inactive, agent_name) = {
            let agent = self
                .agents
                .entry(payload.session_id.clone())
                .or_insert_with(|| AgentInfo {
                    id: payload.session_id.clone(),
                    name: derive_name(
                        payload.transcript_path.as_deref(),
                        payload.cwd.as_deref(),
                        &payload.session_id,
                    ),
                    state: AgentState::Working,
                    task: String::new(),
                    elapsed: 0,
                    tokens: 0,
                    started_at: now,
                    alarm_stopped: true,
                    alarm_error: true,
                    alarm_inactive: true,
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
                agent.alarm_stopped,
                agent.alarm_error,
                agent.alarm_inactive,
                agent.name.clone(),
            )
        };

        // Prune oldest Stopped/Inactive agents when the cap is exceeded
        if self.agents.len() > MAX_AGENTS {
            let mut pruneable: Vec<(u64, String)> = self
                .agents
                .values()
                .filter(|a| matches!(a.state, AgentState::Stopped | AgentState::Inactive))
                .map(|a| (a.started_at, a.id.clone()))
                .collect();
            pruneable.sort_unstable_by_key(|&(t, _)| t);
            let excess = self.agents.len().saturating_sub(MAX_AGENTS);
            for (_, id) in pruneable.iter().take(excess) {
                self.agents.remove(id);
            }
        }

        let sound = if self.dnd {
            None
        } else {
            sound_for_transition(&old_state, &current_state, alarm_stopped, alarm_error, alarm_inactive)
        };

        let alert = make_alert(
            &payload.session_id,
            &agent_name,
            &old_state,
            &current_state,
            alarm_stopped,
            alarm_error,
            alarm_inactive,
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
    // Claude Code has no "thinking" hook — it only fires around tool use and
    // lifecycle events. So Working is our best signal for "reasoning between
    // tools", Running for "a tool is executing", and the rest are lifecycle.
    match p.hook_event_name.as_str() {
        "UserPromptSubmit" => AgentState::Working,   // prompt submitted, Claude reasoning
        "PreToolUse"       => AgentState::Running,   // a tool is executing
        "PostToolUse"      => AgentState::Working,   // back to reasoning about the result
        "Stop"             => AgentState::Stopped,   // finished this turn (fires the "done" chime)
        "SubagentStop"     => AgentState::Working,   // a subagent finished; main agent continues
        "Notification" => {
            // Claude Code sends two very different Notifications:
            //   1. a permission request ("Claude needs your permission to use X")
            //      — a real block, it needs YOU now.
            //   2. an idle nudge ("Claude is waiting for your input") ~60s after a
            //      turn finished — the turn is already done, this is not urgent.
            // Mapping (2) to Waiting was wrong: it bumped a finished agent from
            // "Done" to "Needs you". Keep (2) as Done; only (1) is Waiting.
            let msg = p.message.as_deref().unwrap_or("").to_ascii_lowercase();
            if msg.contains("compact") {
                AgentState::Compacting
            } else if is_idle_nudge(&msg) {
                AgentState::Stopped // finished its turn, just waiting for you — Done
            } else {
                AgentState::Waiting // permission request / other — needs you now
            }
        }
        _ => AgentState::Working,
    }
}

/// True for the "Claude is waiting for your input" idle nudge (fired ~60s after a
/// turn ends), as opposed to a permission request which is an active block.
fn is_idle_nudge(msg_lower: &str) -> bool {
    (msg_lower.contains("waiting") && msg_lower.contains("input"))
        || msg_lower.contains("waiting for your")
}

fn infer_task(p: &HookPayload) -> String {
    match p.hook_event_name.as_str() {
        "UserPromptSubmit" => "Reading your prompt...".to_string(),
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
        "Stop" => "Finished — awaiting your next prompt".to_string(),
        "Notification" => {
            let lower = p.message.as_deref().unwrap_or("").to_ascii_lowercase();
            if lower.contains("compact") {
                "Compacting context...".to_string()
            } else if is_idle_nudge(&lower) {
                "Finished — awaiting your next prompt".to_string()
            } else {
                // Surface the actual notification (e.g. the permission request).
                match p.message.as_deref() {
                    Some(m) if !m.trim().is_empty() => m.trim()[..m.trim().len().min(80)].to_string(),
                    _ => "Needs your input".to_string(),
                }
            }
        }
        _ => String::new(),
    }
}

fn sound_for_transition(
    old: &AgentState,
    new: &AgentState,
    alarm_stopped: bool,
    alarm_error: bool,
    alarm_inactive: bool,
) -> Option<SoundKind> {
    if old == new {
        return None;
    }
    match new {
        AgentState::Stopped  if alarm_stopped  => Some(SoundKind::Done),   // turn complete
        AgentState::Waiting  if alarm_error    => Some(SoundKind::Urgent), // needs your input
        AgentState::Error    if alarm_error    => Some(SoundKind::Urgent),
        AgentState::Inactive if alarm_inactive => Some(SoundKind::Done),
        _ => None,
    }
}

fn make_alert(
    session_id: &str,
    agent_name: &str,
    old: &AgentState,
    new: &AgentState,
    alarm_stopped: bool,
    alarm_error: bool,
    alarm_inactive: bool,
) -> Option<AlertInfo> {
    if old == new {
        return None;
    }
    let enabled = match new {
        AgentState::Stopped  => alarm_stopped,
        AgentState::Waiting  => alarm_error,
        AgentState::Error    => alarm_error,
        AgentState::Inactive => alarm_inactive,
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
        AgentState::Working    => "Thinking",
        AgentState::Running    => "Running",
        AgentState::Compacting => "Compacting",
        AgentState::Waiting    => "Needs you",
        AgentState::Inactive   => "Idle",
        AgentState::Stopped    => "Done",
        AgentState::Error      => "Error",
    }
}

fn derive_name(transcript_path: Option<&str>, cwd: Option<&str>, session_id: &str) -> String {
    // Preferred: the project directory encoded into the transcript path.
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
    // Fallback: the working directory's basename.
    if let Some(dir) = cwd {
        let base = dir.trim_end_matches('/').rsplit('/').next().unwrap_or("");
        if !base.is_empty() {
            return base.to_string();
        }
    }
    // Last resort: a short slice of the session id (no more random animals).
    format!("session {}", &session_id[..session_id.len().min(8)])
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
