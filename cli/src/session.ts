import { CliError } from "./errors.js";

interface AgentFsLike {
  fs?: {
    writeFile?: (path: string, content: string) => Promise<void>;
  };
  kv?: {
    set?: (key: string, value: unknown) => Promise<void>;
  };
}

export interface CommandSession {
  writeInput(command: string, payload: unknown): Promise<void>;
  writeOutput(command: string, payload: unknown): Promise<void>;
  writeError(command: string, payload: unknown): Promise<void>;
}

export async function openSession(sessionId: string): Promise<CommandSession> {
  const normalized = sessionId.trim() || "default";

  let AgentFS: { open: (opts: { id: string }) => Promise<AgentFsLike> } | undefined;
  try {
    const mod = (await import("agentfs-sdk")) as {
      AgentFS?: { open: (opts: { id: string }) => Promise<AgentFsLike> };
      default?: { open: (opts: { id: string }) => Promise<AgentFsLike> };
    };
    AgentFS = mod.AgentFS ?? mod.default;
  } catch {
    throw new CliError(
      "INTERNAL_ERROR",
      "agentfs-sdk is not installed. Run: cd cli && npm install",
    );
  }

  if (!AgentFS?.open) {
    throw new CliError("INTERNAL_ERROR", "agentfs-sdk loaded but AgentFS.open is unavailable");
  }

  const agent = await AgentFS.open({ id: normalized });

  return {
    async writeInput(command, payload) {
      await persist(agent, normalized, command, "input", payload);
    },
    async writeOutput(command, payload) {
      await persist(agent, normalized, command, "output", payload);
    },
    async writeError(command, payload) {
      await persist(agent, normalized, command, "error", payload);
    },
  };
}

async function persist(
  agent: AgentFsLike,
  sessionId: string,
  command: string,
  phase: "input" | "output" | "error",
  payload: unknown,
): Promise<void> {
  const ts = new Date().toISOString().replace(/[:.]/g, "-");
  const safeCmd = command.replace(/[^a-zA-Z0-9:_-]/g, "_");
  const path = `/amctl/${sessionId}/commands/${ts}_${safeCmd}_${phase}.json`;
  const body = JSON.stringify(payload, null, 2);

  if (agent.fs?.writeFile) {
    await agent.fs.writeFile(path, body);
  }

  if (agent.kv?.set) {
    await agent.kv.set(`amctl/${sessionId}/latest/${safeCmd}/${phase}`, payload);
  }
}
