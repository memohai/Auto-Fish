import { CliError } from "./errors.js";
import type { CliContext } from "./types.js";

export interface GlobalOptions {
  baseUrl: string;
  token?: string;
  timeoutMs: number;
  sessionId: string;
}

export function toContext(opts: GlobalOptions): CliContext {
  const baseUrl = (opts.baseUrl || "").trim();
  if (!baseUrl) {
    throw new CliError("INVALID_PARAMS", "baseUrl is required");
  }

  const timeoutMs = Number(opts.timeoutMs);
  if (!Number.isFinite(timeoutMs) || timeoutMs < 100) {
    throw new CliError("INVALID_PARAMS", "timeoutMs must be >= 100");
  }

  return {
    baseUrl,
    token: opts.token?.trim() || undefined,
    timeoutMs,
    sessionId: opts.sessionId?.trim() || "default",
  };
}
