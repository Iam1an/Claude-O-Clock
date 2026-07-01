export type AgentState =
  | "working"
  | "running"
  | "compacting"
  | "waiting"
  | "inactive"
  | "stopped"
  | "error";

export type Tab = "overview" | "agents" | "alerts" | "settings";

export interface Agent {
  id: string;
  name: string;
  state: AgentState;
  task: string;
  elapsed: number; // seconds
  tokens: number;
  startedAt: number; // unix timestamp (seconds)
  alarmStopped: boolean;
  alarmError: boolean;
  alarmInactive: boolean;
}

export interface AlertEntry {
  id: string;
  agentId: string;
  agentName: string;
  state: AgentState;
  timestamp: number;
  message: string;
}

export interface AgentEvent {
  id: number;
  sessionId: string;
  eventType: string;
  toolName?: string;
  description?: string;
  timestamp: number; // unix ms
}

export const STATE_LABEL: Record<AgentState, string> = {
  working:    "Thinking",
  running:    "Running",
  compacting: "Compacting",
  waiting:    "Needs you",
  inactive:   "Idle",
  stopped:    "Done",
  error:      "Error",
};

export const STATE_COLOR: Record<AgentState, string> = {
  working:    "#9B59B6",
  running:    "#E67E22",
  compacting: "#F39C12",
  waiting:    "#EAB308",
  inactive:   "#7F8C8D",
  stopped:    "#27AE60",
  error:      "#E74C3C",
};
