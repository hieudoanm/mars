export interface NotebookTemplate {
  id: string;
  name: string;
  description: string;
  /** Serialized `.mars` notebook JSON used to seed an untitled document. */
  content: string;
}

interface TemplateCell {
  cell_type: 'markdown' | 'code' | 'raw';
  source: string;
}

const cell = (cellType: TemplateCell['cell_type'], source: string): TemplateCell => ({
  cell_type: cellType,
  source,
});

const notebookJson = (title: string, cells: TemplateCell[]): string =>
  JSON.stringify(
    {
      format_version: 2,
      metadata: { language: 'rust', title },
      cells: cells.map((entry, index) => ({
        id: `cell-${String(index + 1).padStart(3, '0')}`,
        cell_type: entry.cell_type,
        source: entry.source,
        metadata: {},
        ...(entry.cell_type === 'code' ? { execution_count: null, outputs: [] } : {}),
      })),
    },
    null,
    2,
  );

export const NOTEBOOK_TEMPLATES: NotebookTemplate[] = [
  {
    id: 'blank',
    name: 'Blank notebook',
    description: 'Start with an empty notebook.',
    content: notebookJson('Untitled', [cell('markdown', '# Untitled\n\nStart writing here.')]),
  },
  {
    id: 'getting-started',
    name: 'Getting started',
    description: 'The Mars tour: markdown, Rust cells, variables, and rich output.',
    content: notebookJson('Getting started', [
      cell(
        'markdown',
        '# Getting started\n\nMars notebooks mix **Markdown**, executable Rust cells, and rich outputs. Variables persist across cells.',
      ),
      cell('code', 'let greeting = "Hello from Mars";\ngreeting'),
      cell('code', 'let numbers = vec![1, 2, 3, 4, 5];\nnumbers.iter().sum::<i32>()'),
      cell(
        'markdown',
        '## Next steps\n\n- Press `Shift+Enter` to run a cell and move on.\n- Open the command palette for Mars commands.\n- Try `mars_prompt("Your name?")` in a cell for interactive input.',
      ),
    ]),
  },
  {
    id: 'data-analysis',
    name: 'Data analysis',
    description: 'Explore a small dataset with vectors, iteration, and formatted output.',
    content: notebookJson('Data analysis', [
      cell('markdown', '# Data analysis\n\nA starting point for exploring numeric data in Rust.'),
      cell(
        'code',
        'let values = vec![12.5, 7.0, 23.0, 4.5, 18.0];\nprintln!("count: {}", values.len());',
      ),
      cell(
        'code',
        'let total: f64 = values.iter().sum();\nlet mean = total / values.len() as f64;\nprintln!("mean: {:.2}", mean);',
      ),
      cell('code', 'let squared: Vec<f64> = values.iter().map(|v| v * v).collect();\nsquared'),
    ]),
  },
];
