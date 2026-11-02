export type MimeBundleData = Record<string, unknown>;

export type SelectedMime =
  | { kind: "image"; mimeType: string; value: string }
  | { kind: "html"; value: string }
  | { kind: "latex"; value: string }
  | { kind: "markdown"; value: string }
  | { kind: "text"; value: string }
  | { kind: "chart"; value: string };

const IMAGE_MIME_TYPES = [
  "image/svg+xml",
  "image/png",
  "image/jpeg",
  "image/gif",
  "image/webp",
];

const CHART_MIME_TYPES = [
  "application/vnd.vega.v5+json",
  "application/vnd.vega-lite.v5+json",
  "application/vnd.vega-lite+json",
];

export function mimeValueToString(value: unknown): string | null {
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

export function isImageMime(mimeType: string): boolean {
  return IMAGE_MIME_TYPES.includes(mimeType);
}

export function selectMime(data: MimeBundleData): SelectedMime | null {
  for (const mimeType of IMAGE_MIME_TYPES) {
    const value = mimeValueToString(data[mimeType]);
    if (value !== null) return { kind: "image", mimeType, value };
  }

  for (const mimeType of CHART_MIME_TYPES) {
    const value = data[mimeType];
    if (value !== undefined) {
      return {
        kind: "chart",
        value: typeof value === "string" ? value : JSON.stringify(value),
      };
    }
  }

  const html = mimeValueToString(data["text/html"]);
  if (html !== null) return { kind: "html", value: html };

  const latex = mimeValueToString(data["text/latex"]);
  if (latex !== null) return { kind: "latex", value: latex };

  const markdown = mimeValueToString(data["text/markdown"]);
  if (markdown !== null) return { kind: "markdown", value: markdown };

  const text = mimeValueToString(data["text/plain"]);
  if (text !== null) return { kind: "text", value: text };

  return null;
}

export function imageDataUri(mimeType: string, value: string): string {
  const data = value.trim();
  if (data.startsWith("data:")) return data;

  if (mimeType === "image/svg+xml") {
    const payload = data.startsWith("<") ? utf8ToBase64(data) : data;
    return `data:image/svg+xml;base64,${payload}`;
  }

  return `data:${mimeType};base64,${data}`;
}

function utf8ToBase64(value: string): string {
  const bytes = new TextEncoder().encode(value);
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}
