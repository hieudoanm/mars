const looksLikeCsv = (text: string): boolean => {
  const lines = text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
  if (lines.length < 2) return false;
  const firstCommas = (lines[0].match(/,/g) ?? []).length;
  if (firstCommas === 0) return false;
  return lines.every((line) => Math.abs((line.match(/,/g) ?? []).length - firstCommas) <= 1);
};

const rawString = (text: string): string => {
  let hashes = '#';
  while (text.includes(`"${hashes}`)) hashes += '#';
  return `r${hashes}"${text}"${hashes}`;
};

export const csvSnippet = (text: string): string | null => {
  const trimmed = text.trim();
  if (!looksLikeCsv(trimmed)) return null;
  return `mars_csv(${rawString(trimmed)});`;
};
