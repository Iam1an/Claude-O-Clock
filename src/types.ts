export type AgentState =
  | "thinking"
  | "planning"
  | "doing"
  | "compacting"
  | "done"
  | "error"
  | "waiting";

export type Tab = "overview" | "agents" | "alerts" | "settings";

export interface Agent {
  id: string;
  name: string;
  state: AgentState;
  task: string;
  elapsed: number; // seconds
  tokens: number;
  startedAt: number; // unix timestamp (seconds)
  alarmDone: boolean;
  alarmError: boolean;
  alarmWaiting: boolean;
}

export interface AlertEntry {
  id: string;
  agentId: string;
  agentName: string;
  state: AgentState;
  timestamp: number;
  message: string;
}

export const STATE_LABEL: Record<AgentState, string> = {
  thinking: "Thinking",
  planning: "Planning",
  doing: "Doing",
  compacting: "Compacting",
  done: "Done",
  error: "Error",
  waiting: "Waiting",
};

export const STATE_COLOR: Record<AgentState, string> = {
  thinking: "#9B59B6",
  planning: "#3498DB",
  doing: "#E67E22",
  compacting: "#F39C12",
  done: "#27AE60",
  error: "#E74C3C",
  waiting: "#7F8C8D",
};
