import { z } from "zod";
import { findNodes, getScreen, getTopActivity } from "../api.js";
import { CliError } from "../errors.js";
import { assert } from "../output.js";
import { toContext, type GlobalOptions } from "../context.js";

const textContainsSchema = z.object({
  text: z.string().min(1),
  ignoreCase: z.boolean().default(true),
});

const topActivitySchema = z.object({
  expected: z.string().min(1),
  mode: z.enum(["contains", "equals"]).default("contains"),
});

const nodeExistsSchema = z.object({
  by: z.enum(["id", "text", "desc", "class", "resource_id"]),
  value: z.string().min(1),
  exactMatch: z.boolean().default(false),
});

export async function runVerify(
  options: GlobalOptions,
  action:
    | { type: "text-contains"; text: string; ignoreCase: boolean }
    | { type: "top-activity"; expected: string; mode: "contains" | "equals" }
    | { type: "node-exists"; by: "id" | "text" | "desc" | "class" | "resource_id"; value: string; exactMatch: boolean },
): Promise<Record<string, unknown>> {
  const ctx = toContext(options);
  if (!ctx.token) {
    throw new CliError("INVALID_PARAMS", "token is required for verify commands");
  }

  switch (action.type) {
    case "text-contains": {
      const p = textContainsSchema.parse(action);
      const screen = await getScreen(ctx);
      const haystack = p.ignoreCase ? screen.toLowerCase() : screen;
      const needle = p.ignoreCase ? p.text.toLowerCase() : p.text;
      const matched = haystack.includes(needle);
      assert(matched, `text not found in screen: ${p.text}`);
      return { verify: "text-contains", matched, text: p.text };
    }

    case "top-activity": {
      const p = topActivitySchema.parse(action);
      const top = await getTopActivity(ctx);
      const matched = p.mode === "equals" ? top === p.expected : top.includes(p.expected);
      assert(matched, `top activity mismatch: expected ${p.mode} ${p.expected}, got ${top}`);
      return { verify: "top-activity", matched, expected: p.expected, actual: top, mode: p.mode };
    }

    case "node-exists": {
      const p = nodeExistsSchema.parse(action);
      const resultText = await findNodes(ctx, p.by, p.value, p.exactMatch);
      const matched = !resultText.startsWith("No nodes found");
      assert(matched, `node not found: by=${p.by}, value=${p.value}`);
      return {
        verify: "node-exists",
        matched,
        by: p.by,
        value: p.value,
        exactMatch: p.exactMatch,
        resultText,
      };
    }
  }
}
