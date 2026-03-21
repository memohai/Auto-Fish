import { z } from "zod";
import { launchApp, pressBack, pressHome, stopApp, swipe, tap, typeText } from "../api.js";
import { CliError } from "../errors.js";
import { toContext, type GlobalOptions } from "../context.js";

const tapSchema = z.object({ x: z.number(), y: z.number() });
const swipeSchema = z.object({
  x1: z.number(),
  y1: z.number(),
  x2: z.number(),
  y2: z.number(),
  duration: z.number().int().positive().default(300),
});
const textSchema = z.object({ text: z.string().min(1) });
const packageSchema = z.object({ packageName: z.string().min(1) });

export async function runAct(
  options: GlobalOptions,
  action:
    | { type: "tap"; x: number; y: number }
    | { type: "swipe"; x1: number; y1: number; x2: number; y2: number; duration: number }
    | { type: "back" }
    | { type: "home" }
    | { type: "text"; text: string }
    | { type: "launch"; packageName: string }
    | { type: "stop"; packageName: string },
): Promise<Record<string, unknown>> {
  const ctx = toContext(options);
  if (!ctx.token) {
    throw new CliError("INVALID_PARAMS", "token is required for act commands");
  }

  switch (action.type) {
    case "tap": {
      const p = tapSchema.parse(action);
      const result = await tap(ctx, p.x, p.y);
      return { action: "tap", result };
    }
    case "swipe": {
      const p = swipeSchema.parse(action);
      const result = await swipe(ctx, p.x1, p.y1, p.x2, p.y2, p.duration);
      return { action: "swipe", result };
    }
    case "back": {
      const result = await pressBack(ctx);
      return { action: "back", result };
    }
    case "home": {
      const result = await pressHome(ctx);
      return { action: "home", result };
    }
    case "text": {
      const p = textSchema.parse(action);
      const result = await typeText(ctx, p.text);
      return { action: "text", result };
    }
    case "launch": {
      const p = packageSchema.parse(action);
      const result = await launchApp(ctx, p.packageName);
      return { action: "launch", result };
    }
    case "stop": {
      const p = packageSchema.parse(action);
      const result = await stopApp(ctx, p.packageName);
      return { action: "stop", result };
    }
  }
}
