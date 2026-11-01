import type { Diagnostic } from "@codemirror/lint";
import type { NotebookCell } from "./types";

export function diagnosticsForCell(cell: NotebookCell): Diagnostic[] {
  return cell.outputs.flatMap((output) =>
    output.type === "error"
      ? parseDiagnostics(cell.source, output.message)
      : [],
  );
}

function parseDiagnostics(source: string, message: string): Diagnostic[] {
  const lines = source.split("\n");
  const diagnostics: Diagnostic[] = [];
  for (const diagnosticLine of message.split("\n")) {
    const match = diagnosticLine.match(
      /^\s*-->\s+[^:]+:(\d+):(\d+)(?:-(\d+):(\d+))?(?:\s+\[(?:primary|secondary)\])?\s*(.*)$/,
    );
    if (!match) continue;
    const lineStart = Number(match[1]);
    const columnStart = Number(match[2]);
    const lineEnd = Number(match[3] ?? lineStart);
    const columnEnd = Number(match[4] ?? columnStart + 1);
    const from = sourceOffset(lines, lineStart, columnStart);
    const to = Math.max(from + 1, sourceOffset(lines, lineEnd, columnEnd));
    diagnostics.push({
      from: Math.min(source.length, from),
      to: Math.min(source.length, to),
      severity: diagnosticLine.includes("warning") ? "warning" : "error",
      message: match[5] || message,
    });
  }
  if (diagnostics.length > 0) return diagnostics;

  const location = message.match(/:(\d+):(\d+)/);
  if (!location) {
    return [{ from: 0, to: Math.max(1, source.length), severity: "error", message }];
  }
  const from = sourceOffset(lines, Number(location[1]), Number(location[2]));
  return [{
    from: Math.min(source.length, from),
    to: Math.min(source.length, from + 1),
    severity: message.includes("warning") ? "warning" : "error",
    message,
  }];
}

function sourceOffset(lines: string[], line: number, column: number): number {
  const lineIndex = Math.max(0, Math.min(lines.length - 1, line - 1));
  const lineStart = lines
    .slice(0, lineIndex)
    .reduce((total, value) => total + value.length + 1, 0);
  const content = lines[lineIndex] ?? "";
  const scalarColumn = Math.max(0, Math.min(column - 1, [...content].length));
  return lineStart + [...content].slice(0, scalarColumn).join("").length;
}

export function parseExecutionTimeout(value: string): { value: number | null; error: string | null } {
  if (value.trim() === "") return { value: null, error: null };
  const timeout = Number(value);
  return Number.isSafeInteger(timeout) && timeout > 0
    ? { value: timeout, error: null }
    : { value: null, error: "Execution limit must be a positive whole number" };
}
