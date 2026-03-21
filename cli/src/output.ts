import { CliError, toCliError } from "./errors.js";
import type { CliCode, CliResult } from "./types.js";

export function ok<T>(command: string, data: T): never {
  print({
    ok: true,
    code: "OK",
    command,
    data,
    timestamp: new Date().toISOString(),
  });
  process.exit(0);
}

export function fail(command: string, err: unknown): never {
  const cliError = toCliError(err);
  const result: CliResult = {
    ok: false,
    code: cliError.code,
    command,
    error: cliError.message,
    timestamp: new Date().toISOString(),
  };
  print(result);
  process.exit(exitCode(cliError.code));
}

function print(result: CliResult): void {
  process.stdout.write(`${JSON.stringify(result)}\n`);
}

function exitCode(code: CliCode): number {
  switch (code) {
    case "INVALID_PARAMS":
      return 2;
    case "AUTH_ERROR":
      return 3;
    case "NETWORK_ERROR":
      return 4;
    case "SERVER_ERROR":
      return 5;
    case "ASSERTION_FAILED":
      return 6;
    case "INTERNAL_ERROR":
      return 10;
    case "OK":
      return 0;
  }
}

export function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new CliError("ASSERTION_FAILED", message);
  }
}
