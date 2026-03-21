import { z } from "zod";
import { launchApp, pressBack, pressHome } from "../api.js";
import { CliError } from "../errors.js";
import { toContext, type GlobalOptions } from "../context.js";

const backSchema = z.object({ times: z.number().int().min(1).max(10).default(1) });
const relaunchSchema = z.object({ packageName: z.string().min(1) });

export async function runRecover(
  options: GlobalOptions,
  action: { type: "back"; times: number } | { type: "home" } | { type: "relaunch"; packageName: string },
): Promise<Record<string, unknown>> {
  const ctx = toContext(options);
  if (!ctx.token) {
    throw new CliError("INVALID_PARAMS", "token is required for recover commands");
  }

  switch (action.type) {
    case "back": {
      const p = backSchema.parse(action);
      for (let i = 0; i < p.times; i += 1) {
        await pressBack(ctx);
      }
      return { recover: "back", times: p.times };
    }
    case "home": {
      await pressHome(ctx);
      return { recover: "home" };
    }
    case "relaunch": {
      const p = relaunchSchema.parse(action);
      await pressHome(ctx);
      const launchResult = await launchApp(ctx, p.packageName);
      return { recover: "relaunch", packageName: p.packageName, launchResult };
    }
  }
}
