import { useCallback, useEffect, useMemo, useState, type FC } from 'react';
import { useEditorPreferences } from './hooks/useEditorPreferences';
import NotebookSession, { type SessionPreferences } from './NotebookSession';
import FileExplorer from './components/explorer/FileExplorer';
import WorkspaceHeader, { type WorkspaceTab } from './components/header/WorkspaceHeader';
import CommandPalette, { type PaletteCommand } from './components/palette/CommandPalette';
import SettingsDialog, {
  applyTheme,
  readThemePreference,
  type ThemePreference,
} from './components/settings/SettingsDialog';
import {
  readLastNotebookPath,
  readRecentPaths,
  readUntitledNotebook,
  readWorkspaceTabs,
  untitledTabKey,
  writeLastNotebookPath,
  writeRecentPath,
  writeWorkspaceTabs,
} from './lib/workspace';
import TemplatePickerDialog from './components/dialogs/TemplatePickerDialog';
import type { NotebookTemplate } from './lib/notebookTemplates';
import type { Notebook } from './types/types';

type SessionEntry = {
  id: string;
  initialPath: string | null;
  untitledId: string | null;
  initialNotebook?: Notebook;
};

const sessionKey = (session: Pick<SessionEntry, 'initialPath' | 'untitledId'>, path: string | null) =>
  path ?? (session.untitledId ? untitledTabKey(session.untitledId) : null);

const App: FC = () => {
  const initialSessions = useMemo<SessionEntry[]>(() => {
    const saved = readWorkspaceTabs();
    if (saved && saved.entries.length > 0) {
      return saved.entries.map((entry) => ({
        id: crypto.randomUUID(),
        initialPath: entry.path,
        untitledId: entry.untitledId ?? (entry.path ? null : crypto.randomUUID()),
        initialNotebook:
          entry.untitledId && !entry.path
            ? ((readUntitledNotebook(entry.untitledId) as Notebook | null) ?? undefined)
            : undefined,
      }));
    }
    const lastPath = readLastNotebookPath();
    return [
      {
        id: crypto.randomUUID(),
        initialPath: lastPath,
        untitledId: lastPath ? null : crypto.randomUUID(),
      },
    ];
  }, []);
  const [sessions, setSessions] = useState<SessionEntry[]>(initialSessions);
  const [activeId, setActiveId] = useState(() => {
    const saved = readWorkspaceTabs();
    if (saved?.activeKey) {
      const match = initialSessions.find((session) => {
        const key = sessionKey(session, session.initialPath);
        return key === saved.activeKey;
      })?.id;
      if (match) return match;
    }
    return initialSessions[0]?.id ?? '';
  });
  const [sessionPaths, setSessionPaths] = useState<Record<string, string | null>>({});
  const [dirtySessions, setDirtySessions] = useState<Record<string, boolean>>({});
  const [pendingCloseId, setPendingCloseId] = useState<string | null>(null);
  const [explorerOpen, setExplorerOpen] = useState(true);
  const [workspaceStatus, setWorkspaceStatus] = useState('');
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [templatePickerOpen, setTemplatePickerOpen] = useState(false);
  const [recents, setRecents] = useState<string[]>(readRecentPaths);
  const [theme, setTheme] = useState<ThemePreference>(readThemePreference);
  const [sessionCommands, setSessionCommands] = useState<Record<string, PaletteCommand[]>>({});
  const preferences = useEditorPreferences();
  const sessionPreferences: SessionPreferences = {
    fontSize: preferences.fontSize,
    updateFontSize: preferences.updateFontSize,
    autocomplete: preferences.preferences.autocomplete,
    folding: preferences.preferences.folding,
    colorScheme: theme === 'mars-light' ? 'light' : 'dark',
  };

  const addSession = useCallback(
    (initialPath: string | null = null, initialNotebook?: Notebook) => {
      const id = crypto.randomUUID();
      setSessions((current) => [
        ...current,
        {
          id,
          initialPath,
          untitledId: initialPath ? null : crypto.randomUUID(),
          initialNotebook,
        },
      ]);
      setActiveId(id);
      return id;
    },
    [],
  );

  const removeSession = useCallback((id: string) => {
    setSessions((current) => {
      const index = current.findIndex((session) => session.id === id);
      if (index < 0) return current;
      const next = current.filter((session) => session.id !== id);
      setActiveId((active) => {
        if (active !== id) return active;
        return next[Math.min(index, next.length - 1)]?.id ?? '';
      });
      return next;
    });
    setSessionPaths((current) => {
      const next = { ...current };
      delete next[id];
      return next;
    });
    setDirtySessions((current) => {
      const next = { ...current };
      delete next[id];
      return next;
    });
  }, []);

  const reorderSessions = useCallback((fromId: string, toId: string) => {
    setSessions((current) => {
      const fromIndex = current.findIndex((session) => session.id === fromId);
      const toIndex = current.findIndex((session) => session.id === toId);
      if (fromIndex < 0 || toIndex < 0 || fromIndex === toIndex) return current;
      const next = [...current];
      const [moved] = next.splice(fromIndex, 1);
      next.splice(toIndex, 0, moved);
      return next;
    });
  }, []);

  const requestCloseSession = useCallback(
    (id: string) => {
      if (dirtySessions[id]) setPendingCloseId(id);
      else removeSession(id);
    },
    [dirtySessions, removeSession],
  );

  const openPathInWorkspace = useCallback(
    (path: string) => {
      const existing = Object.entries(sessionPaths).find(([, p]) => p === path)?.[0];
      if (existing) {
        setActiveId(existing);
        return;
      }
      addSession(path);
    },
    [sessionPaths, addSession],
  );

  const handlePathChange = useCallback((sessionId: string, path: string | null) => {
    setSessionPaths((current) =>
      current[sessionId] === path ? current : { ...current, [sessionId]: path },
    );
    if (path) {
      writeLastNotebookPath(path);
      writeRecentPath(path);
      setRecents(readRecentPaths());
    }
  }, []);

  const handleDirtyChange = useCallback((sessionId: string, isDirty: boolean) => {
    setDirtySessions((current) =>
      current[sessionId] === isDirty ? current : { ...current, [sessionId]: isDirty },
    );
  }, []);

  useEffect(() => {
    const activePath = activeId ? sessionPaths[activeId] : null;
    if (activePath) writeLastNotebookPath(activePath);
  }, [activeId, sessionPaths]);

  const pathsHydrated = sessions.every((session) => session.id in sessionPaths);
  useEffect(() => {
    if (!pathsHydrated) return;
    const entries = sessions
      .map((session) => {
        const path = sessionPaths[session.id] ?? session.initialPath;
        const untitledId = path ? null : session.untitledId;
        const key = path ?? (untitledId ? untitledTabKey(untitledId) : null);
        return key ? { key, path, untitledId } : null;
      })
      .filter((entry): entry is NonNullable<typeof entry> => entry !== null);
    const activeSession = sessions.find((session) => session.id === activeId);
    const activePath = activeId ? (sessionPaths[activeId] ?? null) : null;
    const activeKey = activeSession
      ? sessionKey(activeSession, activePath)
      : null;
    writeWorkspaceTabs({
      entries,
      activeKey: activeKey && entries.some((entry) => entry.key === activeKey)
        ? activeKey
        : (entries[0]?.key ?? null),
    });
  }, [pathsHydrated, sessions, sessionPaths, activeId]);

  useEffect(() => {
    applyTheme(theme);
  }, [theme]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const modifier = event.metaKey || event.ctrlKey;
      if (modifier && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        setPaletteOpen((current) => !current);
      }
      if (modifier && event.key === ',') {
        event.preventDefault();
        setSettingsOpen((current) => !current);
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, []);

  useEffect(() => {
    if (!('__TAURI_INTERNALS__' in window)) return;
    let unlisten: (() => void) | undefined;
    void import('@tauri-apps/api/webview').then(({ getCurrentWebview }) => {
      void getCurrentWebview()
        .onDragDropEvent((event) => {
          if (event.payload.type !== 'drop') return;
          for (const path of event.payload.paths) {
            if (path.endsWith('.mars') || path.endsWith('.ipynb')) {
              openPathInWorkspace(path);
            }
          }
        })
        .then((dispose) => {
          unlisten = dispose;
        });
    });
    return () => unlisten?.();
  }, [openPathInWorkspace]);

  const registerCommands = useCallback((sessionId: string, commands: PaletteCommand[]) => {
    setSessionCommands((current) => ({ ...current, [sessionId]: commands }));
  }, []);

  const paletteCommands: PaletteCommand[] = [
    { id: 'workspace-new-tab', title: 'New notebook tab', run: () => addSession(null) },
    {
      id: 'workspace-new-template',
      title: 'New notebook from template…',
      run: () => setTemplatePickerOpen(true),
    },
    ...recents.slice(0, 5).map((path) => ({
      id: `workspace-recent-${path}`,
      title: `Open recent: ${path.split('/').pop()}`,
      hint: path,
      run: () => openPathInWorkspace(path),
    })),
    {
      id: 'workspace-toggle-explorer',
      title: 'Toggle file explorer',
      run: () => setExplorerOpen((c) => !c),
    },
    { id: 'workspace-settings', title: 'Open settings', run: () => setSettingsOpen(true) },
    ...(activeId ? (sessionCommands[activeId] ?? []) : []),
  ];

  const tabs: WorkspaceTab[] = sessions.map((session) => {
    const path = sessionPaths[session.id];
    return {
      id: session.id,
      label: path?.split('/').pop() ?? 'Untitled notebook',
      isDirty: dirtySessions[session.id] ?? false,
    };
  });

  return (
    <main data-component='App' className='flex h-screen flex-col overflow-hidden'>
      <WorkspaceHeader
        tabs={tabs}
        activeId={activeId}
        explorerOpen={explorerOpen}
        status={workspaceStatus}
        onSelectTab={setActiveId}
        onCloseTab={requestCloseSession}
        onNewTab={() => addSession(null)}
        onToggleExplorer={() => setExplorerOpen((current) => !current)}
        onOpenSettings={() => setSettingsOpen(true)}
        recents={recents}
        onOpenRecent={openPathInWorkspace}
        onReorderTabs={reorderSessions}
      />
      <section className='flex min-h-0 flex-1 overflow-hidden'>
        <FileExplorer
          currentFilePath={activeId ? (sessionPaths[activeId] ?? null) : null}
          open={explorerOpen}
          onOpenNotebook={openPathInWorkspace}
          onStatus={setWorkspaceStatus}
        />
        <div className='relative min-h-0 min-w-0 flex-1'>
          {sessions.map((session) => (
            <div
              key={session.id}
              className={session.id === activeId ? 'h-full' : 'hidden'}
              aria-hidden={session.id !== activeId}>
              <NotebookSession
                sessionId={session.id}
                initialPath={session.initialPath}
                untitledId={session.untitledId}
                initialNotebook={session.initialNotebook}
                isActive={session.id === activeId}
                tabCloseRequested={pendingCloseId === session.id}
                preferences={sessionPreferences}
                onPathChange={handlePathChange}
                onDirtyChange={handleDirtyChange}
                onTabClosed={(id) => {
                  setPendingCloseId(null);
                  removeSession(id);
                }}
                onTabCloseCancel={() => setPendingCloseId(null)}
                registerCommands={registerCommands}
                onOpenSettings={() => setSettingsOpen(true)}
              />
            </div>
          ))}
        </div>
      </section>
      <TemplatePickerDialog
        open={templatePickerOpen}
        onSelect={(template: NotebookTemplate) => {
          setTemplatePickerOpen(false);
          addSession(null, template.notebook);
        }}
        onClose={() => setTemplatePickerOpen(false)}
      />
      <CommandPalette
        open={paletteOpen}
        commands={paletteCommands}
        onClose={() => setPaletteOpen(false)}
      />
      <SettingsDialog
        open={settingsOpen}
        theme={theme}
        onThemeChange={setTheme}
        editorPreferences={preferences.preferences}
        onEditorPreferencesChange={preferences.setPreferences}
        onClose={() => setSettingsOpen(false)}
      />
    </main>
  );
};
App.displayName = 'App';
export default App;
