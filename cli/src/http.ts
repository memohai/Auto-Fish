import { CliError } from "./errors.js";
import type { CliContext } from "./types.js";

interface HttpRequest {
  method: "GET" | "POST";
  path: string;
  body?: unknown;
}

export async function httpJson(ctx: CliContext, req: HttpRequest): Promise<unknown> {
  const url = joinUrl(ctx.baseUrl, req.path);
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), ctx.timeoutMs);

  try {
    const res = await fetch(url, {
      method: req.method,
      headers: {
        "Content-Type": "application/json",
        ...(ctx.token ? { Authorization: `Bearer ${ctx.token}` } : {}),
      },
      body: req.body === undefined ? undefined : JSON.stringify(req.body),
      signal: controller.signal,
    });

    const text = await res.text();
    const parsed = safeJsonParse(text);

    if (res.status === 401) {
      throw new CliError("AUTH_ERROR", "Unauthorized: invalid or missing bearer token");
    }

    if (!res.ok) {
      const message = extractErrorMessage(parsed) ?? `${res.status} ${res.statusText}`;
      throw new CliError("SERVER_ERROR", message);
    }

    return parsed ?? text;
  } catch (err) {
    if (err instanceof CliError) {
      throw err;
    }
    if (err instanceof Error && err.name === "AbortError") {
      throw new CliError("NETWORK_ERROR", `Request timeout after ${ctx.timeoutMs}ms`);
    }
    throw new CliError("NETWORK_ERROR", err instanceof Error ? err.message : String(err));
  } finally {
    clearTimeout(timer);
  }
}

function safeJsonParse(text: string): unknown {
  const trimmed = text.trim();
  if (!trimmed) return null;
  try {
    return JSON.parse(trimmed);
  } catch {
    return trimmed;
  }
}

function extractErrorMessage(data: unknown): string | null {
  if (!data || typeof data !== "object") return null;
  const record = data as Record<string, unknown>;
  if (typeof record.error === "string" && record.error.length > 0) return record.error;
  if (typeof record.message === "string" && record.message.length > 0) return record.message;
  return null;
}

function joinUrl(baseUrl: string, path: string): string {
  const base = baseUrl.replace(/\/$/, "");
  const p = path.startsWith("/") ? path : `/${path}`;
  return `${base}${p}`;
}
