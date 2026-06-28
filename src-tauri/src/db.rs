use rusqlite::{params, Connection, Result};

use crate::types::{AgentInfo, AgentState, AlertInfo};

pub fn init(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;

         CREATE TABLE IF NOT EXISTS agents (
             id          TEXT PRIMARY KEY,
             name        TEXT NOT NULL,
             state       TEXT NOT NULL,
             task        TEXT NOT NULL DEFAULT '',
             elapsed     INTEGER NOT NULL DEFAULT 0,
             tokens      INTEGER NOT NULL DEFAULT 0,
             started_at  INTEGER NOT NULL,
             alarm_done    INTEGER NOT NULL DEFAULT 1,
             alarm_error   INTEGER NOT NULL DEFAULT 1,
             alarm_waiting INTEGER NOT NULL DEFAULT 1
         );

         CREATE TABLE IF NOT EXISTS alerts (
             id          TEXT PRIMARY KEY,
             agent_id    TEXT NOT NULL,
             agent_name  TEXT NOT NULL,
             state       TEXT NOT NULL,
             timestamp   INTEGER NOT NULL,
             message     TEXT NOT NULL
         );",
    )
}

pub fn upsert_agent(conn: &Connection, agent: &AgentInfo) -> Result<()> {
    conn.execute(
        "INSERT INTO agents
             (id, name, state, task, elapsed, tokens, started_at, alarm_done, alarm_error, alarm_waiting)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
         ON CONFLICT(id) DO UPDATE SET
             name        = excluded.name,
             state       = excluded.state,
             task        = excluded.task,
             elapsed     = excluded.elapsed,
             tokens      = excluded.tokens,
             alarm_done    = excluded.alarm_done,
             alarm_error   = excluded.alarm_error,
             alarm_waiting = excluded.alarm_waiting",
        params![
            agent.id,
            agent.name,
            state_to_str(&agent.state),
            agent.task,
            agent.elapsed,
            agent.tokens,
            agent.started_at,
            agent.alarm_done as i64,
            agent.alarm_error as i64,
            agent.alarm_waiting as i64,
        ],
    )?;
    Ok(())
}

pub fn load_agents(conn: &Connection) -> Result<Vec<AgentInfo>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, state, task, elapsed, tokens, started_at,
                alarm_done, alarm_error, alarm_waiting
         FROM agents ORDER BY started_at",
    )?;

    let rows = stmt.query_map([], |row| {
        let state_str: String = row.get(2)?;
        Ok(AgentInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            state: str_to_state(&state_str),
            task: row.get(3)?,
            elapsed: row.get(4)?,
            tokens: row.get(5)?,
            started_at: row.get(6)?,
            alarm_done: row.get::<_, i64>(7)? != 0,
            alarm_error: row.get::<_, i64>(8)? != 0,
            alarm_waiting: row.get::<_, i64>(9)? != 0,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn save_alert(conn: &Connection, alert: &AlertInfo) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO alerts (id, agent_id, agent_name, state, timestamp, message)
         VALUES (?1,?2,?3,?4,?5,?6)",
        params![
            alert.id,
            alert.agent_id,
            alert.agent_name,
            state_to_str(&alert.state),
            alert.timestamp as i64,
            alert.message,
        ],
    )?;
    Ok(())
}

pub fn load_alerts(conn: &Connection) -> Result<Vec<AlertInfo>> {
    let mut stmt = conn.prepare(
        "SELECT id, agent_id, agent_name, state, timestamp, message
         FROM alerts ORDER BY timestamp DESC LIMIT 100",
    )?;

    let rows = stmt.query_map([], |row| {
        let state_str: String = row.get(3)?;
        Ok(AlertInfo {
            id: row.get(0)?,
            agent_id: row.get(1)?,
            agent_name: row.get(2)?,
            state: str_to_state(&state_str),
            timestamp: row.get::<_, i64>(4)? as u64,
            message: row.get(5)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn clear_alerts(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM alerts", [])?;
    Ok(())
}

fn state_to_str(state: &AgentState) -> &'static str {
    match state {
        AgentState::Thinking => "thinking",
        AgentState::Planning => "planning",
        AgentState::Doing => "doing",
        AgentState::Compacting => "compacting",
        AgentState::Done => "done",
        AgentState::Error => "error",
        AgentState::Waiting => "waiting",
    }
}

fn str_to_state(s: &str) -> AgentState {
    match s {
        "planning" => AgentState::Planning,
        "doing" => AgentState::Doing,
        "compacting" => AgentState::Compacting,
        "done" => AgentState::Done,
        "error" => AgentState::Error,
        "waiting" => AgentState::Waiting,
        _ => AgentState::Thinking,
    }
}
