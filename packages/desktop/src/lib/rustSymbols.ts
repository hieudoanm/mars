export type RustSymbol = {
  name: string;
  kind: 'binding' | 'function' | 'type';
};

const BINDING_PATTERN = /\b(?:let|const)\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)/g;
const FUNCTION_PATTERN = /\bfn\s+([A-Za-z_][A-Za-z0-9_]*)/g;
const TYPE_PATTERN = /\b(?:struct|enum|trait|type)\s+([A-Za-z_][A-Za-z0-9_]*)/g;

const collect = (
  source: string,
  pattern: RegExp,
  kind: RustSymbol['kind'],
  into: Map<string, RustSymbol>,
) => {
  for (const match of source.matchAll(pattern)) {
    const name = match[1];
    if (name && !into.has(name)) into.set(name, { name, kind });
  }
};

export const extractRustSymbols = (sources: string[]): RustSymbol[] => {
  const symbols = new Map<string, RustSymbol>();
  for (const source of sources) {
    collect(source, BINDING_PATTERN, 'binding', symbols);
    collect(source, FUNCTION_PATTERN, 'function', symbols);
    collect(source, TYPE_PATTERN, 'type', symbols);
  }
  return [...symbols.values()].sort((a, b) => a.name.localeCompare(b.name));
};
