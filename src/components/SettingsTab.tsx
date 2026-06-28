import { useState } from "react";

interface Settings {
  soundEnabled: boolean;
  notifProvider: "ntfy" | "telegram" | "none";
  ntfyTopic: string;
  mcpEnabled: boolean;
  mcpPort: number;
  maxAgents: number;
}

const DEFAULT_SETTINGS: Settings = {
  soundEnabled: true,
  notifProvider: "ntfy",
  ntfyTopic: "",
  mcpEnabled: true,
  mcpPort: 22361,
  maxAgents: 10,
};

interface SettingsTabProps {
  dnd: boolean;
  onDndChange: (enabled: boolean) => void;
}

export function SettingsTab({ dnd, onDndChange }: SettingsTabProps) {
  const [s, setS] = useState<Settings>(DEFAULT_SETTINGS);

  function toggle(key: keyof Settings) {
    setS((prev) => ({ ...prev, [key]: !prev[key] }));
  }

  return (
    <div>
      <p className="section-header">Alerts</p>
      <div className="card" style={{ padding: "0 12px" }}>
        <div className="settings-row">
          <div>
            <div className="settings-label">Do Not Disturb</div>
            <div className="settings-sublabel">Silence all sounds and notifications</div>
          </div>
          <button
            className={`toggle ${dnd ? "on" : "off"}`}
            onClick={() => onDndChange(!dnd)}
          />
        </div>
        <div className="settings-row">
          <div>
            <div className="settings-label">Sound alerts</div>
            <div className="settings-sublabel">Play sound on Done / Error / Waiting</div>
          </div>
          <button
            className={`toggle ${s.soundEnabled ? "on" : "off"}`}
            onClick={() => toggle("soundEnabled")}
          />
        </div>
      </div>

      <p className="section-header">Notifications</p>
      <div className="card" style={{ padding: "0 12px" }}>
        <div className="settings-row">
          <div className="settings-label">Provider</div>
          <div className="settings-value">{s.notifProvider === "none" ? "Off" : s.notifProvider}</div>
        </div>
        {s.notifProvider === "ntfy" && (
          <div className="settings-row">
            <div className="settings-label">ntfy topic</div>
            <div className="settings-value" style={{ color: s.ntfyTopic ? "var(--text-primary)" : "var(--text-tertiary)" }}>
              {s.ntfyTopic || "not set"}
            </div>
          </div>
        )}
      </div>

      <p className="section-header">MCP Server</p>
      <div className="card" style={{ padding: "0 12px" }}>
        <div className="settings-row">
          <div>
            <div className="settings-label">MCP server</div>
            <div className="settings-sublabel">Expose claudeoclock tools to Claude Code</div>
          </div>
          <button
            className={`toggle ${s.mcpEnabled ? "on" : "off"}`}
            onClick={() => toggle("mcpEnabled")}
          />
        </div>
        <div className="settings-row">
          <div className="settings-label">Port</div>
          <div className="settings-value">{s.mcpPort}</div>
        </div>
      </div>

      <p className="section-header">General</p>
      <div className="card" style={{ padding: "0 12px" }}>
        <div className="settings-row">
          <div className="settings-label">Max agents</div>
          <div className="settings-value">{s.maxAgents}</div>
        </div>
        <div className="settings-row">
          <div className="settings-label">Config file</div>
          <div className="settings-value" style={{ fontSize: 10 }}>~/.config/claudeoclock/config.toml</div>
        </div>
      </div>
    </div>
  );
}
