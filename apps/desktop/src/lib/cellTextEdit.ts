import type { LspCellTextEdit } from './kernelService';

export const offsetToLineChar = (
  source: string,
  offset: number,
): { line: number; character: number } => {
  const clamped = Math.max(0, Math.min(offset, source.length));
  const before = source.slice(0, clamped);
  const line = before.split('\n').length - 1;
  const lineStart = before.lastIndexOf('\n') + 1;
  return { line, character: clamped - lineStart };
};

const lineCharToOffset = (source: string, line: number, character: number): number => {
  const lines = source.split('\n');
  let offset = 0;
  for (let index = 0; index < Math.min(line, lines.length); index += 1) {
    offset += lines[index].length + 1;
  }
  if (line >= lines.length) return source.length;
  return Math.min(offset + character, offset + lines[line].length);
};

/** Apply LSP text edits (cell-local line/character ranges) to a source string. */
export const applyTextEdits = (source: string, edits: LspCellTextEdit[]): string => {
  const ranged = edits
    .map((edit) => ({
      edit,
      start: lineCharToOffset(source, edit.startLine, edit.startCharacter),
      end: lineCharToOffset(source, edit.endLine, edit.endCharacter),
    }))
    .sort((a, b) => b.start - a.start);
  let result = source;
  for (const { edit, start, end } of ranged) {
    result = result.slice(0, start) + edit.newText + result.slice(end);
  }
  return result;
};
