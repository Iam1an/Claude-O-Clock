import type { Agent } from "../types";
import { STATE_COLOR, STATE_LABEL } from "../types";

interface OverviewProps {
  agents: Agent[];
}

export function Overview({ agents }: OverviewProps) {
  const total = agents.length;
  const active = agents.filter((a) =>
    ["working", "running", "compacting"].includes(a.state)
  ).length;
  const done = agents.filter((a) => a.state === "stopped").length;
  const needAttention = agents.filter((a) =>
    ["error", "inactive"].includes(a.state)
  ).length;

  const stateMap = new Map<string, number>();
  agents.forEach((a) => stateMap.set(a.state, (stateMap.get(a.state) ?? 0) + 1));
  const byState = Array.from(stateMap.entries());

  return (
    <div>
      <p className="section-header">At a Glance</p>

      <div className="stat-grid">
        <div className="stat-card">
          <div className="stat-value">{total}</div>
          <div className="stat-label">Total agents</div>
        </div>
        <div className="stat-card">
          <div className="stat-value" style={{ color: "#E67E22" }}>
            {active}
          </div>
          <div className="stat-label">Active</div>
        </div>
        <div className="stat-card">
          <div className="stat-value" style={{ color: "#27AE60" }}>
            {done}
          </div>
          <div className="stat-label">Stopped</div>
        </div>
        <div className="stat-card">
          <div
            className="stat-value"
            style={{ color: needAttention > 0 ? "#E74C3C" : "var(--text-primary)" }}
          >
            {needAttention}
          </div>
          <div className="stat-label">Need attention</div>
        </div>
      </div>

      {byState.length > 0 && (
        <>
          <p className="section-header" style={{ marginTop: 18 }}>
            By State
          </p>
          <div className="card" style={{ padding: "4px 0" }}>
            {byState.map(([state, count]) => (
              <div
                key={state}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 10,
                  padding: "8px 12px",
                  borderBottom: "1px solid var(--border)",
                }}
              >
                <span
                  style={{
                    width: 8,
                    height: 8,
                    borderRadius: "50%",
                    background: STATE_COLOR[state as keyof typeof STATE_COLOR] ?? "#666",
                    flexShrink: 0,
                  }}
                />
                <span style={{ fontSize: 13, color: "var(--text-primary)", flex: 1 }}>
                  {STATE_LABEL[state as keyof typeof STATE_LABEL] ?? state}
                </span>
                <span
                  style={{
                    fontSize: 13,
                    fontWeight: 600,
                    color: "var(--text-secondary)",
                    fontVariantNumeric: "tabular-nums",
                  }}
                >
                  {count}
                </span>
              </div>
            ))}
          </div>
        </>
      )}

      {agents.length === 0 && (
        <div className="empty-state" style={{ marginTop: 24 }}>
          <p className="empty-text">No agents yet. Head to the Agents tab to spawn one.</p>
        </div>
      )}
    </div>
  );
}
