/// Cell-level comments stored in `cell.metadata.comments` — an array of
/// `{ text, created_at }` objects. Metadata is free-form in the `.mars`
/// format, so no schema change is needed and unknown keys round-trip.

export interface CellComment {
  text: string;
  created_at: string;
}

const isCellComment = (value: unknown): value is CellComment =>
  typeof value === 'object' &&
  value !== null &&
  typeof (value as CellComment).text === 'string' &&
  typeof (value as CellComment).created_at === 'string';

export const getCellComments = (metadata: Record<string, unknown> | undefined): CellComment[] => {
  const raw = metadata?.comments;
  if (!Array.isArray(raw)) return [];
  return raw.filter(isCellComment);
};

export const addCellComment = (comments: CellComment[], text: string): CellComment[] => {
  const trimmed = text.trim();
  if (!trimmed) return comments;
  return [...comments, { text: trimmed, created_at: new Date().toISOString() }];
};

export const removeCellComment = (comments: CellComment[], index: number): CellComment[] =>
  comments.filter((_, position) => position !== index);
