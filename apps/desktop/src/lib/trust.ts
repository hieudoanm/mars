const STORAGE_KEY = 'mars.trusted.notebooks';

const readTrusted = (): Set<string> => {
  try {
    const stored: unknown = JSON.parse(window.localStorage.getItem(STORAGE_KEY) ?? '[]');
    return new Set(
      Array.isArray(stored) ? stored.filter((v): v is string => typeof v === 'string') : [],
    );
  } catch {
    return new Set();
  }
};

export const isNotebookTrusted = (path: string): boolean => readTrusted().has(path);

export const trustNotebook = (path: string): void => {
  const trusted = readTrusted();
  trusted.add(path);
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify([...trusted]));
};

export const notebookHasCode = (content: string): boolean => {
  try {
    const parsed: unknown = JSON.parse(content);
    if (typeof parsed !== 'object' || parsed === null) return false;
    const cells = (parsed as { cells?: unknown }).cells;
    return (
      Array.isArray(cells) &&
      cells.some(
        (cell) =>
          typeof cell === 'object' &&
          cell !== null &&
          (cell as { cell_type?: unknown }).cell_type === 'code',
      )
    );
  } catch {
    return false;
  }
};
