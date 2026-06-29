import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Agent, AlertEntry, Tab } from "./types";
import { useAgents } from "./hooks/useAgents";
import { TabBar } from "./components/TabBar";
import { Overview } from "./components/Overview";
import { AgentGrid } from "./components/AgentGrid";
import { AlertsTab } from "./components/AlertsTab";
import { SettingsTab } from "./components/SettingsTab";
import { AgentDetail } from "./components/AgentDetail";

const IS_TAURI = "__TAURI_INTERNALS__" in window;

const TAB_ORDER: Tab[] = ["overview", "agents", "alerts", "settings"];

async function minimize() {
  try { await getCurrentWindow().minimize(); } catch { /* browser dev */ }
}

async function close() {
  try { await getCurrentWindow().close(); } catch { /* browser dev */ }
}

export default function App() {
  const [tab, setTab] = useState<Tab>("agents");
  const [slideDir, setSlideDir] = useState<"left" | "right">("right");
  const [dnd, setDnd] = useState(false);
  const [alerts, setAlerts] = useState<AlertEntry[]>([]);
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);

  const agents = useAgents();

  function handleTabSelect(next: Tab) {
    if (next === tab) return;
    const from = TAB_ORDER.indexOf(tab);
    const to = TAB_ORDER.indexOf(next);
    setSlideDir(to > from ? "right" : "left");
    setTab(next);
  }

  function handleDndChange(next: boolean) {
    setDnd(next);
    if (IS_TAURI) invoke("set_dnd", { enabled: next }).catch(console.error);
  }

  // Load persisted alerts on mount and subscribe to new ones in real time
  useEffect(() => {
    if (!IS_TAURI) return;
    invoke<AlertEntry[]>("get_alerts").then(setAlerts).catch(console.error);
    const unlisten = listen<AlertEntry>("alert-new", (e) => {
      setAlerts((prev) => [e.payload, ...prev].slice(0, 100));
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  function handleClearAlerts() {
    setAlerts([]);
    if (IS_TAURI) invoke("clear_alerts").catch(console.error);
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100vh" }}>
      <header className="titlebar">
        <div className="titlebar-drag" data-tauri-drag-region>
          <span className="titlebar-dot" />
          <span className="titlebar-name">Claude'O'Clock</span>
        </div>

        <div className="titlebar-actions">
          <button
            className={`icon-btn ${dnd ? "active" : ""}`}
            title={dnd ? "Do Not Disturb: on" : "Do Not Disturb: off"}
            onClick={() => handleDndChange(!dnd)}
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              {dnd ? (
                <>
                  <path d="M17.5 17.5A7 7 0 0 1 6.5 6.5" />
                  <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
                </>
              ) : (
                <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
              )}
            </svg>
          </button>

          <button className="icon-btn" title="Spawn agent">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="12" y1="5" x2="12" y2="19" />
              <line x1="5" y1="12" x2="19" y2="12" />
            </svg>
          </button>

          <button className="icon-btn" title="Minimize" onClick={minimize}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
              <line x1="5" y1="12" x2="19" y2="12" />
            </svg>
          </button>

          <button className="icon-btn" title="Close" onClick={close}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
      </header>

      {dnd && (
        <div style={{
          background: "rgba(230,126,34,0.12)",
          borderBottom: "1px solid rgba(230,126,34,0.2)",
          padding: "6px 14px",
          fontSize: 11,
          color: "var(--accent)",
          fontWeight: 500,
          display: "flex",
          alignItems: "center",
          gap: 6,
        }}>
          <span>●</span> Do Not Disturb — sounds silenced, alerts still logged
        </div>
      )}

      {selectedAgentId ? (() => {
        const agent = agents.find(a => a.id === selectedAgentId);
        return agent ? (
          <main className="content" style={{ padding: 0 }}>
            <AgentDetail agent={agent} onBack={() => setSelectedAgentId(null)} />
          </main>
        ) : null;
      })() : (
        <>
          <main className="content">
            <div key={tab} className={`tab-panel tab-panel--${slideDir}`}>
              {tab === "overview" && <Overview agents={agents} />}
              {tab === "agents" && <AgentGrid agents={agents} onSelect={(a: Agent) => setSelectedAgentId(a.id)} />}
              {tab === "alerts" && <AlertsTab alerts={alerts} onClear={handleClearAlerts} />}
              {tab === "settings" && <SettingsTab dnd={dnd} onDndChange={handleDndChange} />}
            </div>
          </main>
          <TabBar active={tab} onSelect={handleTabSelect} alertCount={alerts.length} />
        </>
      )}
    </div>
  );
}
