import { beforeEach, describe, expect, it } from 'vitest';
import {
  hasStoredWorkspaceTabs,
  readLastNotebookPath,
  readRecentPaths,
  readSessionState,
  readUntitledNotebook,
  readWorkspaceTabs,
  untitledTabKey,
  WORKSPACE_TABS_VERSION,
  writeLastNotebookPath,
  writeRecentPath,
  writeSessionState,
  writeUntitledNotebook,
  writeWorkspaceTabs,
} from '../workspace';

describe('workspace persistence', () => {
  beforeEach(() => window.localStorage.clear());

  it('reads and writes the last notebook path', () => {
    expect(readLastNotebookPath()).toBeNull();
    writeLastNotebookPath('/tmp/a.mars');
    expect(readLastNotebookPath()).toBe('/tmp/a.mars');
    writeLastNotebookPath(null);
    expect(readLastNotebookPath()).toBeNull();
  });

  it('stores session restore state per path', () => {
    expect(readSessionState('/tmp/a.mars')).toBeNull();
    writeSessionState('/tmp/a.mars', {
      scrollTop: 120,
      selectedCellId: 'c2',
      collapsedCellIds: ['c1'],
    });
    expect(readSessionState('/tmp/a.mars')).toEqual({
      scrollTop: 120,
      selectedCellId: 'c2',
      collapsedCellIds: ['c1'],
    });
    writeSessionState('/tmp/b.mars', { scrollTop: 0, selectedCellId: null });
    expect(readSessionState('/tmp/b.mars')?.collapsedCellIds).toBeUndefined();
  });

  it('ignores malformed session state', () => {
    window.localStorage.setItem('mars.workspace.session./tmp/a.mars', '{bad');
    expect(readSessionState('/tmp/a.mars')).toBeNull();
    window.localStorage.setItem('mars.workspace.session./tmp/a.mars', '42');
    expect(readSessionState('/tmp/a.mars')).toBeNull();
  });

  it('tracks recent paths most-recent-first without duplicates', () => {
    expect(readRecentPaths()).toEqual([]);
    writeRecentPath('/tmp/a.mars');
    writeRecentPath('/tmp/b.mars');
    writeRecentPath('/tmp/a.mars');
    expect(readRecentPaths()).toEqual(['/tmp/a.mars', '/tmp/b.mars']);
  });

  it('caps recents at eight entries', () => {
    for (let i = 0; i < 10; i += 1) writeRecentPath(`/tmp/f${i}.mars`);
    const recents = readRecentPaths();
    expect(recents).toHaveLength(8);
    expect(recents[0]).toBe('/tmp/f9.mars');
    expect(recents).not.toContain('/tmp/f0.mars');
  });

  it('filters non-string entries from stored recents', () => {
    window.localStorage.setItem('mars.workspace.recentFiles', '["/tmp/a.mars", 7, null]');
    expect(readRecentPaths()).toEqual(['/tmp/a.mars']);
  });

  it('round-trips workspace tabs with a version stamp', () => {
    expect(readWorkspaceTabs()).toBeNull();
    writeWorkspaceTabs({
      entries: [{ key: '/tmp/a.mars', path: '/tmp/a.mars', untitledId: null }],
      activeKey: '/tmp/a.mars',
    });
    expect(readWorkspaceTabs()).toEqual({
      entries: [{ key: '/tmp/a.mars', path: '/tmp/a.mars', untitledId: null }],
      activeKey: '/tmp/a.mars',
    });
    expect(hasStoredWorkspaceTabs()).toBe(true);
  });

  it('rejects unversioned or wrong-version tab state', () => {
    window.localStorage.setItem('mars.workspace.tabs', '{"entries":[]}');
    expect(readWorkspaceTabs()).toBeNull();
    window.localStorage.setItem(
      'mars.workspace.tabs',
      JSON.stringify({ version: WORKSPACE_TABS_VERSION - 1, entries: [] }),
    );
    expect(readWorkspaceTabs()).toBeNull();
    expect(hasStoredWorkspaceTabs()).toBe(true);
  });

  it('parses legacy path-list tab state', () => {
    window.localStorage.setItem(
      'mars.workspace.tabs',
      JSON.stringify({
        version: WORKSPACE_TABS_VERSION,
        paths: ['/tmp/a.mars', 3],
        activePath: '/tmp/a.mars',
      }),
    );
    expect(readWorkspaceTabs()).toEqual({
      entries: [{ key: '/tmp/a.mars', path: '/tmp/a.mars', untitledId: null }],
      activeKey: '/tmp/a.mars',
    });
  });

  it('drops malformed tab entries but keeps valid ones', () => {
    window.localStorage.setItem(
      'mars.workspace.tabs',
      JSON.stringify({
        version: WORKSPACE_TABS_VERSION,
        entries: [
          { key: '/tmp/a.mars', path: '/tmp/a.mars', untitledId: null },
          'junk',
          { path: null, untitledId: null },
          { untitledId: 'u1' },
        ],
        activeKey: 'untitled:u1',
      }),
    );
    const tabs = readWorkspaceTabs();
    expect(tabs?.entries).toHaveLength(2);
    expect(tabs?.entries[1]).toEqual({
      key: 'untitled:u1',
      path: null,
      untitledId: 'u1',
    });
    expect(tabs?.activeKey).toBe('untitled:u1');
  });

  it('returns null for non-object tab payloads', () => {
    window.localStorage.setItem('mars.workspace.tabs', '"hello"');
    expect(readWorkspaceTabs()).toBeNull();
    window.localStorage.setItem('mars.workspace.tabs', '{bad');
    expect(readWorkspaceTabs()).toBeNull();
  });

  it('round-trips untitled notebooks keyed by id', () => {
    expect(untitledTabKey('abc')).toBe('untitled:abc');
    expect(readUntitledNotebook('abc')).toBeNull();
    const notebook = { format_version: 1, cells: [{ id: 'c1' }] };
    writeUntitledNotebook('abc', notebook);
    expect(readUntitledNotebook('abc')).toEqual(notebook);
  });

  it('rejects untitled payloads without cells', () => {
    window.localStorage.setItem('mars.workspace.untitled.xyz', '{"nope":1}');
    expect(readUntitledNotebook('xyz')).toBeNull();
    window.localStorage.setItem('mars.workspace.untitled.xyz', 'not json');
    expect(readUntitledNotebook('xyz')).toBeNull();
  });
});
