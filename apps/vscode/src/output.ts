import * as vscode from "vscode";

export type MarsOutput =
  | { type: "text"; text: string }
  | { type: "error"; name: string; message: string; traceback: string[] }
  | { type: "html"; html: string }
  | { type: "image"; mime_type: string; data: string }
  | { type: "table"; columns: string[]; rows: string[][] }
  | { type: "latex"; latex: string }
  | { type: "chart"; spec: Record<string, unknown> }
  | {
      type: "mime_bundle";
      data: Record<string, unknown>;
      metadata?: Record<string, unknown>;
    };

const MIME_PRIORITY = [
  "image/svg+xml",
  "image/png",
  "image/jpeg",
  "image/gif",
  "image/webp",
  "text/html",
  "text/latex",
  "text/markdown",
  "text/plain",
];

const CHART_MIME_TYPES = [
  "application/vnd.vega.v5+json",
  "application/vnd.vega-lite.v5+json",
  "application/vnd.vega-lite+json",
];

const ERROR_MIME = "application/vnd.code.notebook.error";

function textItem(value: string, mime: string): vscode.NotebookCellOutputItem {
  return new vscode.NotebookCellOutputItem(Buffer.from(value, "utf-8"), mime);
}

function imageItem(mimeType: string, value: string): vscode.NotebookCellOutputItem {
  const data = value.trim();
  if (data.startsWith("data:")) {
    const uri = parseDataUri(data);
    if (uri) {
      if (mimeType === "image/svg+xml") {
        return textItem(
          uri.base64
            ? Buffer.from(uri.payload, "base64").toString("utf-8")
            : decodeDataUriText(uri.payload),
          mimeType,
        );
      }
      return new vscode.NotebookCellOutputItem(
        Buffer.from(uri.payload, uri.base64 ? "base64" : "utf-8"),
        mimeType,
      );
    }
  }
  if (mimeType === "image/svg+xml") {
    return textItem(
      data.startsWith("<")
        ? data
        : Buffer.from(data, "base64").toString("utf-8"),
      mimeType,
    );
  }
  return new vscode.NotebookCellOutputItem(Buffer.from(data, "base64"), mimeType);
}

function parseDataUri(
  value: string,
): { base64: boolean; payload: string } | null {
  const comma = value.indexOf(",");
  if (comma === -1) return null;
  const header = value.slice(5, comma);
  return {
    base64: header.endsWith(";base64"),
    payload: value.slice(comma + 1),
  };
}

function decodeDataUriText(payload: string): string {
  try {
    return decodeURIComponent(payload);
  } catch {
    return payload;
  }
}

function mimeValueToString(value: unknown): string | null {
  if (typeof value === "string") return value;
  if (Array.isArray(value)) {
    const parts: string[] = [];
    for (const part of value) {
      if (typeof part !== "string") return null;
      parts.push(part);
    }
    return parts.join("");
  }
  return null;
}

function mimeItem(mimeType: string, value: unknown): vscode.NotebookCellOutputItem | null {
  if (mimeType.startsWith("image/")) {
    const text = mimeValueToString(value);
    return text === null ? null : imageItem(mimeType, text);
  }

  const text = mimeValueToString(value);
  if (text !== null) return textItem(text, mimeType);
  return textItem(JSON.stringify(value), mimeType);
}

function tableToHtml(columns: string[], rows: string[][]): string {
  const headers = columns.map((column) => `<th>${escapeHtml(column)}</th>`).join("");
  const body = rows
    .map(
      (row) =>
        `<tr>${row.map((value) => `<td>${escapeHtml(value)}</td>`).join("")}</tr>`,
    )
    .join("");
  return `<table><thead><tr>${headers}</tr></thead><tbody>${body}</tbody></table>`;
}

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

export function marsOutputToNotebookOutput(
  output: MarsOutput,
): vscode.NotebookCellOutput {
  const items: vscode.NotebookCellOutputItem[] = [];

  switch (output.type) {
    case "text":
      items.push(textItem(output.text, "text/plain"));
      break;
    case "error": {
      const error = {
        name: output.name,
        message: output.message,
        stack: output.traceback.join("\n"),
      };
      items.push(textItem(JSON.stringify(error), ERROR_MIME));
      break;
    }
    case "html":
      items.push(textItem(output.html, "text/html"));
      break;
    case "latex":
      items.push(textItem(output.latex, "text/latex"));
      break;
    case "image":
      items.push(imageItem(output.mime_type, output.data));
      break;
    case "table":
      items.push(textItem(tableToHtml(output.columns, output.rows), "text/html"));
      break;
    case "chart":
      items.push(textItem(JSON.stringify(output.spec), "application/vnd.vega-lite+json"));
      break;
    case "mime_bundle": {
      const entries = Object.entries(output.data);
      entries.sort((left, right) => mimeRank(left[0]) - mimeRank(right[0]));
      for (const [mimeType, value] of entries) {
        const item = mimeItem(mimeType, value);
        if (item) items.push(item);
      }
      if (items.length === 0) {
        items.push(textItem(JSON.stringify(output.data), "text/plain"));
      }
      break;
    }
  }

  return new vscode.NotebookCellOutput(items);
}

function mimeRank(mimeType: string): number {
  const index = MIME_PRIORITY.indexOf(mimeType);
  return index === -1 ? MIME_PRIORITY.length : index;
}

export function notebookOutputToMarsOutput(
  output: vscode.NotebookCellOutput,
): MarsOutput {
  const items = output.items;
  if (items.length === 0) {
    return { type: "text", text: "" };
  }
  if (items.length === 1) {
    return itemToMarsOutput(items[0]);
  }

  const data: Record<string, string> = {};
  for (const item of items) {
    const mimeType = item.mime ?? "text/plain";
    if (mimeType === ERROR_MIME) continue;
    data[mimeType] = itemToMarsValue(item);
  }
  return { type: "mime_bundle", data, metadata: {} };
}

function itemToMarsValue(item: vscode.NotebookCellOutputItem): string {
  const mimeType = item.mime ?? "text/plain";
  const bytes = Buffer.from(item.data);
  if (mimeType.startsWith("image/") && mimeType !== "image/svg+xml") {
    return bytes.toString("base64");
  }
  return bytes.toString("utf-8");
}

function itemToMarsOutput(item: vscode.NotebookCellOutputItem): MarsOutput {
  const mimeType = item.mime ?? "text/plain";
  const bytes = Buffer.from(item.data);
  const text = bytes.toString("utf-8");

  if (mimeType === ERROR_MIME) {
    return parseErrorOutput(text);
  }
  if (mimeType.startsWith("image/")) {
    return { type: "image", mime_type: mimeType, data: itemToMarsValue(item) };
  }
  if (mimeType === "text/html") {
    return { type: "html", html: text };
  }
  if (mimeType === "text/latex") {
    return { type: "latex", latex: text };
  }
  if (CHART_MIME_TYPES.includes(mimeType)) {
    return { type: "chart", spec: parseSpec(text) };
  }
  if (mimeType === "text/plain" || mimeType === "text/markdown") {
    return { type: "text", text };
  }
  return { type: "mime_bundle", data: { [mimeType]: text }, metadata: {} };
}

function parseErrorOutput(text: string): MarsOutput {
  try {
    const parsed: unknown = JSON.parse(text);
    if (typeof parsed === "object" && parsed !== null) {
      const record = parsed as Record<string, unknown>;
      const name = typeof record.name === "string" ? record.name : "Error";
      const message = typeof record.message === "string" ? record.message : text;
      const stack = typeof record.stack === "string" ? record.stack : "";
      return {
        type: "error",
        name,
        message,
        traceback: stack ? stack.split("\n") : [],
      };
    }
  } catch {
    // Fall through to a plain error output.
  }
  return { type: "error", name: "Error", message: text, traceback: [] };
}

function parseSpec(text: string): Record<string, unknown> {
  try {
    const parsed: unknown = JSON.parse(text);
    if (typeof parsed === "object" && parsed !== null) {
      return parsed as Record<string, unknown>;
    }
  } catch {
    // Return an empty specification below.
  }
  return {};
}

export function appendOutput(
  output: vscode.NotebookCellOutput,
): vscode.NotebookCellOutput {
  return output;
}
