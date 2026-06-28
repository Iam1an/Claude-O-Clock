import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Agent } from "../types";

const IS_TAURI = "__TAURI_INTERNALS__" in window;

export function useAgents(): Agent[] {
  const [agents, setAgents] = useState<Agent[]>([]);

  useEffect(() => {
    if (!IS_TAURI) return;

    invoke<Agent[]>("get_agents").then(setAgents).catch(console.error);

    const unlistenPromise = listen<Agent[]>("agent-update", (event) => {
      setAgents(event.payload);
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  return agents;
}
