import {
  directoryFromPath,
  fileNameFromPath,
  normalizeNotebookFileName,
  renamedNotebookPath,
} from '../filePath';

describe('filePath', () => {
  it('extracts file names and directories from POSIX and Windows paths', () => {
    expect(fileNameFromPath('/books/demo.mars')).toBe('demo.mars');
    expect(fileNameFromPath('C:\\books\\demo.mars')).toBe('demo.mars');
    expect(directoryFromPath('/books/demo.mars')).toBe('/books/');
    expect(directoryFromPath('C:\\books\\demo.mars')).toBe('C:\\books\\');
    expect(directoryFromPath('demo.mars')).toBe('');
  });

  it('normalizes notebook file names', () => {
    expect(normalizeNotebookFileName('analysis')).toBe('analysis.mars');
    expect(normalizeNotebookFileName('analysis.MARS')).toBe('analysis.MARS');
    expect(normalizeNotebookFileName('  notes  ')).toBe('notes.mars');
    expect(normalizeNotebookFileName('')).toBeNull();
    expect(normalizeNotebookFileName('   ')).toBeNull();
    expect(normalizeNotebookFileName('.')).toBeNull();
    expect(normalizeNotebookFileName('..')).toBeNull();
    expect(normalizeNotebookFileName('../escape')).toBeNull();
    expect(normalizeNotebookFileName('nested/file')).toBeNull();
    expect(normalizeNotebookFileName('nested\\file')).toBeNull();
  });

  it('builds a renamed path in the same directory', () => {
    expect(renamedNotebookPath('/books/demo.mars', 'analysis')).toBe('/books/analysis.mars');
    expect(renamedNotebookPath('C:\\books\\demo.mars', 'analysis.mars')).toBe(
      'C:\\books\\analysis.mars',
    );
    expect(renamedNotebookPath('/books/demo.mars', '../escape')).toBeNull();
  });
});
