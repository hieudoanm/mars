export type MarsOutput =
  | { type: 'text'; text: string }
  | { type: 'error'; name: string; message: string; traceback: string[] }
  | { type: 'html'; html: string }
  | { type: 'image'; mime_type: string; data: string }
  | { type: 'table'; columns: string[]; rows: string[][] }
  | { type: 'latex'; latex: string }
  | { type: 'chart'; spec: Record<string, unknown> }
  | {
      type: 'mime_bundle';
      data: Record<string, unknown>;
      metadata?: Record<string, unknown>;
    };
