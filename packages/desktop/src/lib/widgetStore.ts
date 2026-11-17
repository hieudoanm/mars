import type { CommMessage } from '../types/types';
import { listenKernelStream, sendKernelComm } from './kernelService';

/// Front-end comm registry for Jupyter-style widgets. Comm messages arrive
/// on the `kernel-stream` channel while cells execute; widget controls send
/// `comm_msg` updates back through `send_kernel_comm`.

export interface WidgetEntry {
  commId: string;
  session: string;
  language: string;
  state: Record<string, unknown>;
}

const widgets = new Map<string, WidgetEntry>();
const listeners = new Set<() => void>();
let bridgeStarted = false;

const notify = () => {
  for (const listener of listeners) listener();
};

const asRecord = (value: unknown): Record<string, unknown> | null =>
  typeof value === 'object' && value !== null && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : null;

const applyComm = (message: CommMessage, session: string, language: string) => {
  if (message.comm_type === 'comm_close') {
    if (widgets.delete(message.comm_id)) notify();
    return;
  }
  const data = asRecord(message.data) ?? {};
  if (message.comm_type === 'comm_open') {
    const state = asRecord(data.state) ?? {};
    widgets.set(message.comm_id, { commId: message.comm_id, session, language, state });
    notify();
    return;
  }
  const entry = widgets.get(message.comm_id);
  if (!entry) return;
  const method = data.method;
  let changed = false;
  const patchEntry = (commId: string, patch: Record<string, unknown> | null) => {
    const target = widgets.get(commId);
    if (!target || !patch) return;
    widgets.set(commId, { ...target, state: { ...target.state, ...patch } });
    changed = true;
  };
  if (method === 'update' || method === 'echo_update') {
    patchEntry(message.comm_id, asRecord(data.state));
  } else if (method === 'update_states') {
    const states = asRecord(data.states);
    if (states) {
      for (const [modelId, modelState] of Object.entries(states)) {
        patchEntry(modelId, asRecord(modelState));
      }
    }
  }
  if (changed) notify();
};

export const applyCommMessages = (comms: CommMessage[], session: string, language: string) => {
  for (const comm of comms) applyComm(comm, session, language);
};

export const getWidget = (commId: string): WidgetEntry | undefined => widgets.get(commId);

export const subscribeWidgets = (listener: () => void): (() => void) => {
  listeners.add(listener);
  return () => listeners.delete(listener);
};

/// Registers the persistent kernel-stream listener once. Safe to call
/// repeatedly; outside Tauri it is a no-op.
export const ensureWidgetBridge = () => {
  if (bridgeStarted) return;
  bridgeStarted = true;
  void listenKernelStream((message) => {
    if (message.type === 'comm') {
      applyComm(message.message, message.session, message.language ?? 'rust');
    }
  });
};

/// Sends a front-end → back-end comm message for a widget and applies any
/// comm traffic published in response.
export const sendWidgetComm = async (
  commId: string,
  commType: string,
  data: Record<string, unknown>,
): Promise<void> => {
  ensureWidgetBridge();
  const entry = widgets.get(commId);
  if (!entry) return;
  const comms = await sendKernelComm(entry.session, commId, commType, data, entry.language);
  applyCommMessages(comms, entry.session, entry.language);
};
