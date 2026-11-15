import type { Notebook } from '../types/types';

export type NotebookTemplate = {
  id: string;
  name: string;
  description: string;
  notebook: Notebook;
};

const cell = (
  cellType: 'markdown' | 'code' | 'raw',
  source: string,
): Notebook['cells'][number] => ({
  id: crypto.randomUUID(),
  cell_type: cellType,
  source,
  execution_count: null,
  outputs: [],
  metadata: {},
});

export const NOTEBOOK_TEMPLATES: NotebookTemplate[] = [
  {
    id: 'data-analysis',
    name: 'Data analysis',
    description: 'Explore a small dataset with vectors, iteration, and formatted output.',
    notebook: {
      format_version: 2,
      metadata: { language: 'rust', title: 'Data analysis' },
      cells: [
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
      ],
    },
  },
  {
    id: 'getting-started',
    name: 'Getting started',
    description: 'The Project Mars tour: markdown, Rust cells, variables, and rich output.',
    notebook: {
      format_version: 2,
      metadata: { language: 'rust', title: 'Getting started' },
      cells: [
        cell(
          'markdown',
          '# Getting started\n\nMars notebooks mix **Markdown**, executable Rust cells, and rich outputs. Variables persist across cells.',
        ),
        cell('code', 'let greeting = "Hello from Project Mars";\ngreeting'),
        cell('code', 'let numbers = vec![1, 2, 3, 4, 5];\nnumbers.iter().sum::<i32>()'),
        cell(
          'markdown',
          '## Next steps\n\n- Press `Shift+Enter` to run a cell and move on.\n- Open the command palette with `Cmd+K`.\n- Try `mars_prompt("Your name?")` in a cell for interactive input.',
        ),
      ],
    },
  },
];
