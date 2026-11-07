export type DiffRow = {
  type: 'same' | 'add' | 'del';
  text: string;
};

const lcsTable = (a: string[], b: string[]): number[][] => {
  const table: number[][] = Array.from({ length: a.length + 1 }, () =>
    new Array<number>(b.length + 1).fill(0),
  );
  for (let i = a.length - 1; i >= 0; i -= 1) {
    for (let j = b.length - 1; j >= 0; j -= 1) {
      table[i][j] =
        a[i] === b[j] ? table[i + 1][j + 1] + 1 : Math.max(table[i + 1][j], table[i][j + 1]);
    }
  }
  return table;
};

export const diffLines = (before: string, after: string): DiffRow[] => {
  const a = before.split('\n');
  const b = after.split('\n');
  const table = lcsTable(a, b);
  const rows: DiffRow[] = [];
  let i = 0;
  let j = 0;
  while (i < a.length && j < b.length) {
    if (a[i] === b[j]) {
      rows.push({ type: 'same', text: a[i] });
      i += 1;
      j += 1;
    } else if (table[i + 1][j] >= table[i][j + 1]) {
      rows.push({ type: 'del', text: a[i] });
      i += 1;
    } else {
      rows.push({ type: 'add', text: b[j] });
      j += 1;
    }
  }
  while (i < a.length) {
    rows.push({ type: 'del', text: a[i] });
    i += 1;
  }
  while (j < b.length) {
    rows.push({ type: 'add', text: b[j] });
    j += 1;
  }
  return rows;
};
