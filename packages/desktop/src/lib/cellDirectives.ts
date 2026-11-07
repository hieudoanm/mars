export type CellDirectives = {
  dependencies: [string, string][];
  timeoutMillis: number | null;
};

const DEP_PREFIXES = ['// dep:', '// deps:'];
const TIMEOUT_PREFIX = '// timeout:';

export const parseCellDirectives = (source: string): CellDirectives => {
  const dependencies: [string, string][] = [];
  let timeoutMillis: number | null = null;
  for (const line of source.split('\n')) {
    const trimmed = line.trim();
    const depPrefix = DEP_PREFIXES.find((prefix) => trimmed.startsWith(prefix));
    if (depPrefix) {
      const spec = trimmed.slice(depPrefix.length).trim();
      const at = spec.lastIndexOf('@');
      if (at > 0) {
        dependencies.push([spec.slice(0, at).trim(), spec.slice(at + 1).trim()]);
      } else if (spec) {
        dependencies.push([spec, '*']);
      }
      continue;
    }
    if (trimmed.startsWith(TIMEOUT_PREFIX)) {
      const raw = trimmed.slice(TIMEOUT_PREFIX.length).trim();
      const seconds = raw.endsWith('s') ? Number(raw.slice(0, -1)) : Number(raw);
      if (Number.isFinite(seconds) && seconds > 0) {
        timeoutMillis = Math.round(seconds * 1000);
      }
    }
  }
  return { dependencies, timeoutMillis };
};
