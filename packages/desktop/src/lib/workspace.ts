const LAST_PATH_KEY = 'mars.workspace.lastPath';

export const readLastNotebookPath = (): string | null => {
  try {
    return window.localStorage.getItem(LAST_PATH_KEY);
  } catch {
    return null;
  }
};

export const writeLastNotebookPath = (path: string | null): void => {
  try {
    if (path) window.localStorage.setItem(LAST_PATH_KEY, path);
    else window.localStorage.removeItem(LAST_PATH_KEY);
  } catch {
    // Storage unavailable; session restore is best-effort.
  }
};

const SESSION_STATE_PREFIX = 'mars.workspace.session.';

export type SessionRestoreState = {
  scrollTop: number;
  selectedCellId: string | null;
  collapsedCellIds?: string[];
};

export const readSessionState = (path: string): SessionRestoreState | null => {
  try {
    const raw = window.localStorage.getItem(SESSION_STATE_PREFIX + path);
    if (!raw) return null;
    const parsed: unknown = JSON.parse(raw);
    if (typeof parsed !== 'object' || parsed === null) return null;
    const record = parsed as Record<string, unknown>;
    return {
      scrollTop: typeof record.scrollTop === 'number' ? record.scrollTop : 0,
      selectedCellId: typeof record.selectedCellId === 'string' ? record.selectedCellId : null,
      collapsedCellIds: Array.isArray(record.collapsedCellIds)
        ? record.collapsedCellIds.filter((id): id is string => typeof id === 'string')
        : undefined,
    };
  } catch {
    return null;
  }
};

export const writeSessionState = (path: string, state: SessionRestoreState): void => {
  try {
    window.localStorage.setItem(SESSION_STATE_PREFIX + path, JSON.stringify(state));
  } catch {
    // best-effort
  }
};

const RECENTS_KEY = 'mars.workspace.recentFiles';
const RECENTS_LIMIT = 8;

export const readRecentPaths = (): string[] => {
  try {
    const raw = window.localStorage.getItem(RECENTS_KEY);
    if (!raw) return [];
    const parsed: unknown = JSON.parse(raw);
    return Array.isArray(parsed)
      ? parsed.filter((item): item is string => typeof item === 'string')
      : [];
  } catch {
    return [];
  }
};

export const writeRecentPath = (path: string): void => {
  try {
    const next = [path, ...readRecentPaths().filter((item) => item !== path)].slice(
      0,
      RECENTS_LIMIT,
    );
    window.localStorage.setItem(RECENTS_KEY, JSON.stringify(next));
  } catch {
    // best-effort
  }
};

const TABS_KEY = 'mars.workspace.tabs';
// Bump when the restore contract changes so stale state from older builds is
// treated as a first launch instead of silently reopening a notebook.
export const WORKSPACE_TABS_VERSION = 2;
const UNTITLED_PREFIX = 'mars.workspace.untitled.';

export type WorkspaceTabEntry = {
  key: string;
  path: string | null;
  untitledId: string | null;
};

export type WorkspaceTabs = {
  entries: WorkspaceTabEntry[];
  activeKey: string | null;
};

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null;

const parseEntry = (value: unknown): WorkspaceTabEntry | null => {
  if (!isRecord(value)) return null;
  const path = typeof value.path === 'string' ? value.path : null;
  const untitledId = typeof value.untitledId === 'string' ? value.untitledId : null;
  const key =
    typeof value.key === 'string'
      ? value.key
      : (path ?? (untitledId ? `untitled:${untitledId}` : null));
  if (!key || (!path && !untitledId)) return null;
  return { key, path, untitledId };
};

export const hasStoredWorkspaceTabs = (): boolean => {
  try {
    return window.localStorage.getItem(TABS_KEY) !== null;
  } catch {
    return false;
  }
};

export const readWorkspaceTabs = (): WorkspaceTabs | null => {
  try {
    const raw = window.localStorage.getItem(TABS_KEY);
    if (!raw) return null;
    const parsed: unknown = JSON.parse(raw);
    if (!isRecord(parsed)) return null;
    if (parsed.version !== WORKSPACE_TABS_VERSION) return null;
    if (Array.isArray(parsed.entries)) {
      const entries = parsed.entries
        .map(parseEntry)
        .filter((entry): entry is WorkspaceTabEntry => entry !== null);
      return {
        entries,
        activeKey: typeof parsed.activeKey === 'string' ? parsed.activeKey : null,
      };
    }
    // Legacy format: { paths: string[], activePath: string | null }
    const paths = Array.isArray(parsed.paths)
      ? parsed.paths.filter((item): item is string => typeof item === 'string')
      : [];
    return {
      entries: paths.map((path) => ({ key: path, path, untitledId: null })),
      activeKey: typeof parsed.activePath === 'string' ? parsed.activePath : null,
    };
  } catch {
    return null;
  }
};

export const writeWorkspaceTabs = (tabs: WorkspaceTabs): void => {
  try {
    window.localStorage.setItem(
      TABS_KEY,
      JSON.stringify({ ...tabs, version: WORKSPACE_TABS_VERSION }),
    );
  } catch {
    // best-effort
  }
};

export const untitledTabKey = (untitledId: string): string => `untitled:${untitledId}`;

export const readUntitledNotebook = (untitledId: string): unknown | null => {
  try {
    const raw = window.localStorage.getItem(UNTITLED_PREFIX + untitledId);
    if (!raw) return null;
    const parsed: unknown = JSON.parse(raw);
    return isRecord(parsed) && Array.isArray(parsed.cells) ? parsed : null;
  } catch {
    return null;
  }
};

export const writeUntitledNotebook = (untitledId: string, notebook: unknown): void => {
  try {
    window.localStorage.setItem(UNTITLED_PREFIX + untitledId, JSON.stringify(notebook));
  } catch {
    // best-effort
  }
};
