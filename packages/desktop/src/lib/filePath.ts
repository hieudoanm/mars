export const fileNameFromPath = (path: string): string => path.split(/[\\/]/).pop() ?? path;

export const directoryFromPath = (path: string): string => {
  const index = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
  return index >= 0 ? path.slice(0, index + 1) : '';
};

export const normalizeNotebookFileName = (name: string): string | null => {
  const trimmed = name.trim();
  if (!trimmed || trimmed === '.' || trimmed === '..' || /[\\/]/.test(trimmed)) return null;
  return /\.mars$/i.test(trimmed) ? trimmed : `${trimmed}.mars`;
};

export const renamedNotebookPath = (path: string, fileName: string): string | null => {
  const normalized = normalizeNotebookFileName(fileName);
  if (!normalized) return null;
  return `${directoryFromPath(path)}${normalized}`;
};
