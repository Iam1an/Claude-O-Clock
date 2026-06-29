import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Agent, AgentEvent } from "../types";
import { STATE_COLOR, STATE_LABEL } from "../types";

const IS_TAURI = "__TAURI_INTERNALS__" in window;

function eventColor(eventType: string): string {
  switch (eventType) {
    case "PreToolUse":      return STATE_COLOR.running;
    case "PostToolUse":     return STATE_COLOR.working;
    case "UserPromptSubmit":return STATE_COLOR.working;
    case "Stop":            return STATE_COLOR.inactive;
    case "Notification":    return STATE_COLOR.compacting;
    default:                return STATE_COLOR.working;
  }
}

function eventLabel(e: AgentEvent): string {
  switch (e.eventType) {
    case "PreToolUse":
      return `${e.toolName ?? "tool"}${e.description ? `: ${e.description}` : ""}`;
    case "PostToolUse":
      return `${e.toolName ?? "tool"} finished`;
    case "UserPromptSubmit":
      return "Prompt submitted";
    case "Stop":
      return "Turn complete";
    case "Notification":
      return e.description ?? "Notification";
    default:
      return e.eventType;
  }
}

function eventIcon(eventType: string): string {
  switch (eventType) {
    case "PreToolUse":       return "▶";
    case "PostToolUse":      return "✓";
    case "UserPromptSubmit": return "→";
    case "Stop":             return "■";
    case "Notification":     return "◆";
    default:                 return "·";
  }
}

function relativeTime(tsMs: number): string {
  const diff = Math.floor((Date.now() - tsMs) / 1000);
  if (diff < 5)    return "just now";
  if (diff < 60)   return `${diff}s ago`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  return `${Math.floor(diff / 3600)}h ago`;
}

// Activity bar chart — 15 buckets × 2 min = 30 min window
const BUCKET_COUNT = 15;
const BUCKET_MS = 2 * 60 * 1000;

function ActivityChart({ events }: { events: AgentEvent[] }) {
  const now = Date.now();
  const windowStart = now - BUCKET_COUNT * BUCKET_MS;

  const buckets: { count: number; color: string }[] = Array.from(
    { length: BUCKET_COUNT },
    () => ({ count: 0, color: "rgba(255,255,255,0.06)" })
  );

  for (const e of events) {
    if (e.timestamp < windowStart) continue;
    const idx = Math.floor((e.timestamp - windowStart) / BUCKET_MS);
    if (idx >= 0 && idx < BUCKET_COUNT) {
      buckets[idx].count++;
      buckets[idx].color = eventColor(e.eventType);
    }
  }

  const maxCount = Math.max(...buckets.map(b => b.count), 1);
  const chartH = 52;
  const barW = 14;
  const gap = 3;
  const totalW = BUCKET_COUNT * (barW + gap) - gap;

  return (
    <div>
      <svg width={totalW} height={chartH} style={{ display: "block", width: "100%" }} viewBox={`0 0 ${totalW} ${chartH}`} preserveAspectRatio="none">
        {buckets.map((b, i) => {
          const barH = b.count === 0 ? 3 : Math.max(6, (b.count / maxCount) * (chartH - 6));
          const x = i * (barW + gap);
          const y = chartH - barH;
          return (
            <rect
              key={i}
              x={x} y={y}
              width={barW} height={barH}
              rx={3}
              fill={b.count === 0 ? "rgba(255,255,255,0.06)" : b.color}
              opacity={b.count === 0 ? 1 : 0.85}
            />
          );
        })}
      </svg>
      <div style={{ display: "flex", justifyContent: "space-between", marginTop: 5, fontSize: 10, color: "var(--text-tertiary)" }}>
        <span>30m ago</span>
        <span>now</span>
      </div>
    </div>
  );
}

interface Props {
  agent: Agent;
  onBack: () => void;
}

export function AgentDetail({ agent, onBack }: Props) {
  const [events, setEvents] = useState<AgentEvent[]>([]);
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(agent.name);
  const color = STATE_COLOR[agent.state];

  function fetchEvents() {
    if (!IS_TAURI) return;
    invoke<AgentEvent[]>("get_agent_events", { id: agent.id })
      .then(setEvents)
      .catch(console.error);
  }

  useEffect(() => {
    fetchEvents();
  }, [agent.id]);

  // Refresh event log whenever the agent updates
  useEffect(() => {
    fetchEvents();
  }, [agent.state, agent.task]);

  function commitRename() {
    const name = draft.trim();
    if (name && name !== agent.name && IS_TAURI) {
      invoke("rename_agent", { id: agent.id, name }).catch(console.error);
    }
    setEditing(false);
  }

  function setAlarm(key: "alarmStopped" | "alarmError" | "alarmInactive", val: boolean) {
    if (!IS_TAURI) return;
    invoke("set_agent_alarms", {
      id: agent.id,
      stopped:  key === "alarmStopped"  ? val : agent.alarmStopped,
      error:    key === "alarmError"    ? val : agent.alarmError,
      inactive: key === "alarmInactive" ? val : agent.alarmInactive,
    }).catch(console.error);
  }

  const alarmRows: { key: "alarmStopped" | "alarmError" | "alarmInactive"; label: string; sub: string }[] = [
    { key: "alarmInactive", label: "Inactive",  sub: "Turn complete, waiting for next prompt" },
    { key: "alarmStopped",  label: "Stopped",   sub: "Session ended" },
    { key: "alarmError",    label: "Error",     sub: "Something went wrong" },
  ];

  return (
    <div className="agent-detail">
      {/* Header */}
      <div className="detail-header">
        <button className="icon-btn" onClick={onBack} title="Back">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
            <polyline points="15 18 9 12 15 6" />
          </svg>
        </button>

        <span className="state-dot" style={{ background: color }} />

        {editing ? (
          <input
            className="agent-name-input"
            style={{ fontSize: 13 }}
            value={draft}
            autoFocus
            maxLength={40}
            onChange={e => setDraft(e.target.value)}
            onBlur={commitRename}
            onKeyDown={e => {
              if (e.key === "Enter") commitRename();
              if (e.key === "Escape") { setDraft(agent.name); setEditing(false); }
            }}
          />
        ) : (
          <span
            className="agent-name agent-name--editable"
            style={{ fontSize: 13, flex: 1 }}
            title="Click to rename"
            onClick={() => { setDraft(agent.name); setEditing(true); }}
          >
            {agent.name}
          </span>
        )}

        <span
          className="state-badge"
          style={{ "--state-color": color, "--state-color-dim": `${color}22` } as React.CSSProperties}
        >
          {STATE_LABEL[agent.state]}
        </span>
      </div>

      {/* Scrollable body */}
      <div className="detail-body">

        {/* Activity chart */}
        <p className="section-header">Activity — last 30 min</p>
        <div className="card" style={{ marginBottom: 14 }}>
          <ActivityChart events={events} />
        </div>

        {/* Notification toggles */}
        <p className="section-header">Notifications</p>
        <div className="card" style={{ padding: "0 12px", marginBottom: 14 }}>
          {alarmRows.map(({ key, label, sub }, i) => (
            <div
              key={key}
              className="settings-row"
              style={{
                borderBottom: i < alarmRows.length - 1 ? "1px solid var(--border)" : "none",
              }}
            >
              <div>
                <div className="settings-label" style={{ fontSize: 12 }}>{label}</div>
                <div className="settings-sublabel">{sub}</div>
              </div>
              <button
                className={`toggle ${agent[key] ? "on" : "off"}`}
                onClick={() => setAlarm(key, !agent[key])}
              />
            </div>
          ))}
        </div>

        {/* Audit log */}
        <p className="section-header">Audit Log</p>
        {events.length === 0 ? (
          <div className="empty-state" style={{ padding: "20px 0" }}>
            <p className="empty-text">No events recorded yet</p>
          </div>
        ) : (
          <div className="audit-log">
            {events.map(e => (
              <div key={e.id} className="audit-row">
                <span className="audit-icon" style={{ color: eventColor(e.eventType) }}>
                  {eventIcon(e.eventType)}
                </span>
                <span className="audit-label">{eventLabel(e)}</span>
                <span className="audit-time">{relativeTime(e.timestamp)}</span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
