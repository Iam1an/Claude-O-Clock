import type { AlertEntry } from "../types";
import { STATE_COLOR, STATE_LABEL } from "../types";

interface AlertsTabProps {
  alerts: AlertEntry[];
  onClear: () => void;
}

function formatTime(ts: number): string {
  return new Date(ts).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

export function AlertsTab({ alerts, onClear }: AlertsTabProps) {
  if (alerts.length === 0) {
    return (
      <div className="empty-state">
        <svg className="empty-icon" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
          <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" />
          <path d="M13.73 21a2 2 0 0 1-3.46 0" />
        </svg>
        <p className="empty-text">No alerts yet.</p>
      </div>
    );
  }

  return (
    <div>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 10 }}>
        <p className="section-header" style={{ margin: 0 }}>
          Recent Alerts
        </p>
        <button
          onClick={onClear}
          style={{
            background: "none",
            border: "none",
            fontSize: 11,
            color: "var(--text-tertiary)",
            cursor: "pointer",
            padding: "2px 6px",
            borderRadius: 4,
          }}
        >
          Clear all
        </button>
      </div>

      <div className="alert-list">
        {alerts.map((alert) => (
          <div key={alert.id} className="alert-item">
            <span
              className="alert-state-dot"
              style={{ background: STATE_COLOR[alert.state] }}
            />
            <div className="alert-body">
              <div className="alert-title">
                {alert.agentName} → {STATE_LABEL[alert.state]}
              </div>
              <div className="alert-time">{formatTime(alert.timestamp)}</div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
