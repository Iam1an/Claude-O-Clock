import type { Agent } from "../types";
import { AgentCard } from "./AgentCard";

interface AgentGridProps {
  agents: Agent[];
  onSelect?: (agent: Agent) => void;
}

export function AgentGrid({ agents, onSelect }: AgentGridProps) {
  const twoCol = agents.length > 5;

  if (agents.length === 0) {
    return (
      <div className="empty-state">
        <svg className="empty-icon" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
          <circle cx="12" cy="8" r="4" />
          <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" />
        </svg>
        <p className="empty-text">No agents running.<br />Spawn one to get started.</p>
      </div>
    );
  }

  return (
    <div className={`agent-grid ${twoCol ? "two-col" : ""}`}>
      {agents.map((agent) => (
        <AgentCard key={agent.id} agent={agent} compact={twoCol} onSelect={onSelect} />
      ))}
    </div>
  );
}
