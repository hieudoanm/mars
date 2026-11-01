export type CellType = "markdown" | "code" | "raw";

export type Output =
  | { type: "text"; text: string }
  | { type: "error"; name: string; message: string; traceback: string[] }
  | { type: "html"; html: string }
  | { type: "image"; mime_type: string; data: string }
  | { type: "table"; columns: string[]; rows: string[][] }
  | { type: "latex"; latex: string }
  | { type: "chart"; spec: Record<string, unknown> }
  | { type: "mime_bundle"; data: Record<string, unknown>; metadata: Record<string, unknown> };

export type NotebookCell = {
  id: string;
  cell_type: CellType;
  source: string;
  execution_count: number | null;
  outputs: Output[];
  metadata: Record<string, unknown>;
};

export type Notebook = {
  format_version: number;
  metadata: {
    title?: string;
    language: string;
    dependencies?: Record<string, string>;
    cargo_lock?: string;
    [key: string]: unknown;
  };
  cells: NotebookCell[];
};

export type KernelResponse =
  | {
      type: "execution";
      outputs: Output[];
      execution_count: number;
      duration_millis: number;
    }
  | { type: "reset"; status: string }
  | { type: "status"; status: string; execution_count: number }
  | { type: "error"; message: string };
