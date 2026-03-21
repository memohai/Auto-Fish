import { getHealth, getTopActivity } from "../api.js";
import { CliError } from "../errors.js";
import { toContext, type GlobalOptions } from "../context.js";

export async function runPreflight(options: GlobalOptions): Promise<Record<string, unknown>> {
  const ctx = toContext(options);
  const health = await getHealth(ctx);

  let auth = "skipped";
  let topActivity: string | null = null;

  if (ctx.token) {
    try {
      topActivity = await getTopActivity(ctx);
      auth = "ok";
    } catch (err) {
      if (err instanceof CliError && err.code === "AUTH_ERROR") {
        auth = "unauthorized";
      } else {
        throw err;
      }
    }
  }

  return {
    baseUrl: ctx.baseUrl,
    timeoutMs: ctx.timeoutMs,
    tokenProvided: Boolean(ctx.token),
    health,
    auth,
    topActivity,
  };
}
