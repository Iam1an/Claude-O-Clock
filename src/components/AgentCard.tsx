import type { Agent } from "../types";
import { STATE_COLOR, STATE_LABEL } from "../types";

interface AgentCardProps {
  agent: Agent;
  compact?: boolean;
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

const ACTIVE_STATES = new Set(["thinking", "planning", "doing", "compacting"]);

export function AgentCard({ agent, compact = false }: AgentCardProps) {
  const color = STATE_COLOR[agent.state];
  const label = STATE_LABEL[agent.state];
  const isActive = ACTIVE_STATES.has(agent.state);

  return (
    <div
      className="agent-card"
      style={{ "--state-color": color, "--state-color-dim": `${color}22` } as React.CSSProperties}
    >
      <div className="agent-card-header">
        <span
          className={`state-dot ${isActive ? "pulsing" : ""}`}
          style={{ background: color }}
        />
        <span className="agent-name">{agent.name}</span>
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
