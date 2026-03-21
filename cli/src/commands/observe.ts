import { z } from "zod";
import { getScreen, getScreenshot, getTopActivity } from "../api.js";
import { CliError } from "../errors.js";
import { toContext, type GlobalOptions } from "../context.js";

const screenshotSchema = z.object({
  maxDim: z.number().int().positive().default(700),
  quality: z.number().int().min(1).max(100).default(80),
});

export async function runObserve(
  options: GlobalOptions,
  action: { type: "screen" } | { type: "screenshot"; maxDim: number; quality: number } | { type: "top" },
): Promise<Record<string, unknown>> {
  const ctx = toContext(options);
  if (!ctx.token) {
    throw new CliError("INVALID_PARAMS", "token is required for observe commands");
  }

  switch (action.type) {
    case "screen": {
      const screen = await getScreen(ctx);
      return { observation: "screen", screen };
    }
    case "screenshot": {
      const p = screenshotSchema.parse(action);
      const screenshotBase64 = await getScreenshot(ctx, p.maxDim, p.quality);
      return { observation: "screenshot", screenshotBase64, maxDim: p.maxDim, quality: p.quality };
    }
    case "top": {
      const topActivity = await getTopActivity(ctx);
      return { observation: "top", topActivity };
    }
  }
}
