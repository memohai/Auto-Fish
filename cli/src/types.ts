export type CliCode =
  | "OK"
  | "INVALID_PARAMS"
  | "NETWORK_ERROR"
  | "AUTH_ERROR"
  | "SERVER_ERROR"
  | "ASSERTION_FAILED"
  | "INTERNAL_ERROR";

export interface CliResult<T = unknown> {
  ok: boolean;
  code: CliCode;
  command: string;
  data?: T;
  error?: string;
  timestamp: string;
}

export interface ApiEnvelope {
  ok: boolean;
  data?: string | null;
  error?: string | null;
}

export interface CliContext {
  baseUrl: string;
  token?: string;
  timeoutMs: number;
  sessionId: string;
}
