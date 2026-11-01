export type SearchOptions = {
  caseSensitive: boolean;
  wholeWord: boolean;
  regex: boolean;
};

export const DEFAULT_SEARCH_OPTIONS: SearchOptions = {
  caseSensitive: false,
  wholeWord: false,
  regex: false,
};

const escapeRegExp = (value: string): string => value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

export const createSearchPattern = (query: string, options: SearchOptions): RegExp | null => {
  const trimmed = query.trim();
  if (!trimmed) return null;
  const flags = options.caseSensitive ? 'g' : 'gi';
  if (options.regex) {
    try {
      return new RegExp(trimmed, flags);
    } catch {
      return null;
    }
  }
  const body = options.wholeWord ? `\\b${escapeRegExp(trimmed)}\\b` : escapeRegExp(trimmed);
  return new RegExp(body, flags);
};

export const matchesPattern = (pattern: RegExp, text: string): boolean => {
  pattern.lastIndex = 0;
  return pattern.test(text);
};
