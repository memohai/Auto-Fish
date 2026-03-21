import { z } from "zod";
import { CliError } from "./errors.js";
import { httpJson } from "./http.js";
import type { ApiEnvelope, CliContext } from "./types.js";

const apiEnvelopeSchema = z.object({
  ok: z.boolean(),
  data: z.string().nullable().optional(),
  error: z.string().nullable().optional(),
});

export async function getHealth(ctx: CliContext): Promise<unknown> {
  return httpJson(ctx, { method: "GET", path: "/health" });
}

export async function getScreen(ctx: CliContext): Promise<string> {
  const res = await httpJson(ctx, { method: "GET", path: "/api/screen" });
  return unwrapEnvelope(res, "screen");
}

export async function getScreenshot(ctx: CliContext, maxDim: number, quality: number): Promise<string> {
  const res = await httpJson(ctx, {
    method: "GET",
    path: `/api/screenshot?max_dim=${maxDim}&quality=${quality}`,
  });
  return unwrapEnvelope(res, "screenshot");
}

export async function tap(ctx: CliContext, x: number, y: number): Promise<string> {
  return callAction(ctx, "/api/tap", { x, y }, "tap");
}

export async function swipe(
  ctx: CliContext,
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  duration: number,
): Promise<string> {
  return callAction(ctx, "/api/swipe", { x1, y1, x2, y2, duration }, "swipe");
}

export async function pressBack(ctx: CliContext): Promise<string> {
  return callAction(ctx, "/api/press/back", {}, "press_back");
}

export async function pressHome(ctx: CliContext): Promise<string> {
  return callAction(ctx, "/api/press/home", {}, "press_home");
}

export async function typeText(ctx: CliContext, text: string): Promise<string> {
  return callAction(ctx, "/api/text", { text }, "type_text");
}

export async function launchApp(ctx: CliContext, packageName: string): Promise<string> {
  return callAction(ctx, "/api/app/launch", { package_name: packageName }, "launch_app");
}

export async function stopApp(ctx: CliContext, packageName: string): Promise<string> {
  return callAction(ctx, "/api/app/stop", { package_name: packageName }, "stop_app");
}

export async function getTopActivity(ctx: CliContext): Promise<string> {
  const res = await httpJson(ctx, { method: "GET", path: "/api/app/top" });
  return unwrapEnvelope(res, "top_activity");
}

export async function findNodes(
  ctx: CliContext,
  by: "id" | "text" | "desc" | "class" | "resource_id",
  value: string,
  exactMatch: boolean,
): Promise<string> {
  const normalizedBy = by.toUpperCase();
  const mappedBy = normalizedBy === "RESOURCE_ID" ? "resource_id" : normalizedBy.toLowerCase();
  return callAction(
    ctx,
    "/api/nodes/find",
    {
      by: mappedBy,
      value,
      exact_match: exactMatch,
    },
    "find_nodes",
  );
}

async function callAction(ctx: CliContext, path: string, body: object, op: string): Promise<string> {
  const res = await httpJson(ctx, { method: "POST", path, body });
  return unwrapEnvelope(res, op);
}

function unwrapEnvelope(input: unknown, op: string): string {
  const parsed = apiEnvelopeSchema.safeParse(input);
  if (!parsed.success) {
    throw new CliError("SERVER_ERROR", `Unexpected ${op} response format`);
  }
  const env: ApiEnvelope = parsed.data;
  if (!env.ok) {
    throw new CliError("SERVER_ERROR", env.error || `${op} failed`);
  }
  return env.data ?? "";
}
