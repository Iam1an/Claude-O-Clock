import { useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Agent } from "../types";
import { STATE_COLOR, STATE_LABEL } from "../types";

interface AgentCardProps {
  agent: Agent;
  compact?: boolean;
  onSelect?: (agent: Agent) => void;
}

function formatElapsed(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  return `${h}h ${m}m`;
}

function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
  return String(n);
}

const ACTIVE_STATES = new Set(["working", "running", "compacting"]);
const IS_TAURI = "__TAURI_INTERNALS__" in window;

export function AgentCard({ agent, compact = false, onSelect }: AgentCardProps) {
  const color = STATE_COLOR[agent.state];
  const label = STATE_LABEL[agent.state];
  const isActive = ACTIVE_STATES.has(agent.state);

  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(agent.name);
  const inputRef = useRef<HTMLInputElement>(null);

  function startEdit(e: React.MouseEvent) {
    e.stopPropagation();
    setDraft(agent.name);
    setEditing(true);
    // focus happens via autoFocus on the input
  }

  function commit() {
    const name = draft.trim();
    if (name && name !== agent.name && IS_TAURI) {
      invoke("rename_agent", { id: agent.id, name }).catch(console.error);
    }
    setEditing(false);
  }

  function cancel() {
    setDraft(agent.name);
    setEditing(false);
  }

  function onKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter") { e.preventDefault(); commit(); }
    if (e.key === "Escape") cancel();
  }

  return (
    <div
      className="agent-card"
      style={{ "--state-color": color, "--state-color-dim": `${color}22` } as React.CSSProperties}
      onClick={() => onSelect?.(agent)}
    >
      <div className="agent-card-header">
        <span
          className={`state-dot ${isActive ? "pulsing" : ""}`}
          style={{ background: color }}
        />

        {editing ? (
          <input
            ref={inputRef}
            className="agent-name-input"
            value={draft}
            autoFocus
            maxLength={40}
            onChange={(e) => setDraft(e.target.value)}
            onBlur={commit}
            onKeyDown={onKeyDown}
            onClick={(e) => e.stopPropagation()}
          />
        ) : (
          <span
            className="agent-name agent-name--editable"
            title="Click to rename"
            onClick={startEdit}
          >
            {agent.name}
          </span>
        )}

        <span className="state-badge">{label}</span>
      </div>

      {!compact && (
        <div className="agent-task">{agent.task}</div>
      )}

      <div className="agent-meta">
        <span className="meta-pill">{formatElapsed(agent.elapsed)}</span>
        <span className="meta-sep" />
        <span className="meta-pill">{formatTokens(agent.tokens)} tok</span>
      </div>
    </div>
  );
}
