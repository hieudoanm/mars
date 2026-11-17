import { getDocs, getDoc } from '../docs';
import fs from 'node:fs';

interface MockFs {
  readFileSync: (filePath: string) => string;
  readdirSync: (dir: string, options?: unknown) => unknown[];
  existsSync: (filePath: string) => boolean;
  _setStore: (s: Record<string, string>) => void;
  _clearStore: () => void;
}

vi.mock('node:fs', () => {
  const store = new Map<string, string>();
  const rootMarker = 'md/docs';
  const toRel = (filePath: unknown) => {
    const s = String(filePath).replace(/\\/g, '/');
    const index = s.indexOf(rootMarker);
    const rel = index >= 0 ? s.slice(index + rootMarker.length) : s;
    return rel.replace(/^\/+/, '');
  };
  const dirent = (name: string, isDir: boolean) => ({
    name,
    isDirectory: () => isDir,
    isFile: () => !isDir,
  });
  const mod: MockFs = {
    readFileSync: (filePath: string) => {
      const rel = toRel(filePath);
      if (!store.has(rel)) {
        const err: Error & {
          code?: string;
          errno?: number;
          path?: string;
          syscall?: string;
        } = new Error(`ENOENT: no such file '${rel}'`);
        err.code = 'ENOENT';
        err.errno = -2;
        err.path = rel;
        err.syscall = 'open';
        throw err;
      }
      return store.get(rel)!;
    },
    readdirSync: (dir: string) => {
      const rel = toRel(dir);
      const prefix = rel ? `${rel}/` : '';
      const children = new Map<string, boolean>();
      for (const key of store.keys()) {
        if (!key.startsWith(prefix)) continue;
        const rest = key.slice(prefix.length);
        if (!rest) continue;
        const [head, ...tail] = rest.split('/');
        children.set(head, tail.length > 0 || children.get(head) === true);
      }
      return [...children.entries()].map(([name, isDir]) => dirent(name, isDir));
    },
    existsSync: (filePath: string) => store.has(toRel(filePath)),
    _setStore: (s: Record<string, string>) => {
      store.clear();
      for (const [key, value] of Object.entries(s)) store.set(key, value);
    },
    _clearStore: () => store.clear(),
  };
  return { __esModule: true, default: mod };
});

const mockFs = fs as unknown as MockFs;

beforeEach(() => {
  mockFs._clearStore();
});

const withFrontmatter = (title: string, order?: number, extra = '') =>
  `---\ntitle: ${title}${order !== undefined ? `\norder: ${order}` : ''}\nauthor: Project Mars Team\n---\n# ${title}\n\n${extra}`;

describe('docs lib (mocked fs)', () => {
  it('getDocs returns docs sorted by order', () => {
    mockFs._setStore({
      '01-getting-started.md': withFrontmatter('Getting Started', 1),
      '03-notebook-format.md': withFrontmatter('Notebook Format', 2),
      '02-cli.md': withFrontmatter('CLI', 3),
      'index.md': '---\ntitle: Index\n---\n# Index\n\nindex',
    });
    const docs = getDocs();
    expect(docs.map((d) => d.slug)).toEqual(['01-getting-started', '03-notebook-format', '02-cli']);
  });

  it('getDocs sorts undefined order after defined order', () => {
    mockFs._setStore({
      'zebra.md': withFrontmatter('Zebra', 3),
      'alpha.md': withFrontmatter('Alpha', 1),
      'noorder-a.md': withFrontmatter('No Order A', undefined),
      'noorder-b.md': withFrontmatter('No Order B', undefined),
    });
    const docs = getDocs();
    const slugs = docs.map((d) => d.slug);
    expect(slugs[0]).toBe('alpha');
    expect(slugs[1]).toBe('zebra');
    expect([slugs[2], slugs[3]].sort()).toEqual(['noorder-a', 'noorder-b']);
  });

  it('getDoc returns doc with title from frontmatter', () => {
    mockFs._setStore({ 'cli.md': withFrontmatter('CLI', 3) });
    const doc = getDoc('cli');
    expect(doc).toBeDefined();
    expect(doc?.title).toBe('CLI');
    expect(doc?.order).toBe(3);
  });

  it('getDoc falls back to H1 heading when no frontmatter title', () => {
    mockFs._setStore({
      'runtime.md': `---\ndescription: x\norder: 5\n---\n# Execution runtime\n\nbody text`,
    });
    const doc = getDoc('runtime');
    expect(doc?.title).toBe('Execution runtime');
  });

  it('getDoc falls back to slug-derived title when no title and no heading', () => {
    mockFs._setStore({
      'no-title.md': `---\ndescription: x\n---\nsome body without any heading`,
    });
    const doc = getDoc('no-title');
    expect(doc?.title).toBe('No Title');
  });

  it('getDoc handles content without frontmatter', () => {
    mockFs._setStore({ 'plain.md': 'just plain content\nno dashes here' });
    const doc = getDoc('plain');
    expect(doc).toBeDefined();
    expect(doc?.title).toBe('Plain');
    expect(doc?.content).toContain('plain content');
  });

  it('getDoc handles partial frontmatter', () => {
    mockFs._setStore({ 'partial.md': '---\ntitle: Partial\n' });
    const doc = getDoc('partial');
    expect(doc).toBeDefined();
    expect(doc?.content).toContain('title: Partial');
  });

  it('getDoc returns undefined for invalid slug', () => {
    expect(getDoc('invalid slug')).toBeUndefined();
    expect(getDoc('../secret')).toBeUndefined();
  });

  it('getDoc returns undefined when file does not exist', () => {
    mockFs._setStore({ 'existing.md': withFrontmatter('Existing') });
    expect(getDoc('missing')).toBeUndefined();
  });

  it('getDocs discovers docs inside chapter folders', () => {
    mockFs._setStore({
      'foundations/getting-started.md': withFrontmatter('Getting Started', 1),
      'platform/runtime.md': withFrontmatter('Runtime', 2),
      'index.md': '---\ntitle: Index\n---\n# Index\n\nindex',
    });
    const docs = getDocs();
    expect(docs.map((d) => d.slug)).toEqual(['getting-started', 'runtime']);
    expect(docs[0].chapter).toBe('foundations');
    expect(docs[1].chapter).toBe('platform');
    expect(getDoc('runtime')?.title).toBe('Runtime');
  });

  it('getDocs filters out index.md and non-matching files', () => {
    mockFs._setStore({
      'readme.md': withFrontmatter('Readme'),
      'index.md': '---\ntitle: Index\n---\n# Index\n\nindex',
      'not-a-doc.txt': 'not a doc',
    });
    const docs = getDocs();
    expect(docs.map((d) => d.slug)).toEqual(['readme']);
  });
});
