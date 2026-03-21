import type { CliCode } from "./types.js";

export class CliError extends Error {
  readonly code: CliCode;

  constructor(code: CliCode, message: string) {
    super(message);
    this.name = "CliError";
    this.code = code;
  }
}

export function toCliError(err: unknown): CliError {
  if (err instanceof CliError) {
    return err;
  }
  if (err instanceof Error) {
    return new CliError("INTERNAL_ERROR", err.message);
  }
  return new CliError("INTERNAL_ERROR", String(err));
}
