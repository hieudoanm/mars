import { CompletionContext } from '@codemirror/autocomplete';
import type { Completion, CompletionResult } from '@codemirror/autocomplete';
import { extractRustSymbols } from './rustSymbols';

const KEYWORDS = [
  'as',
  'async',
  'await',
  'break',
  'const',
  'continue',
  'crate',
  'dyn',
  'else',
  'enum',
  'extern',
  'false',
  'fn',
  'for',
  'if',
  'impl',
  'in',
  'let',
  'loop',
  'match',
  'mod',
  'move',
  'mut',
  'pub',
  'ref',
  'return',
  'self',
  'Self',
  'static',
  'struct',
  'super',
  'trait',
  'true',
  'type',
  'unsafe',
  'use',
  'where',
  'while',
];

const PRIMITIVES = [
  'i8',
  'i16',
  'i32',
  'i64',
  'i128',
  'isize',
  'u8',
  'u16',
  'u32',
  'u64',
  'u128',
  'usize',
  'f32',
  'f64',
  'bool',
  'char',
  'str',
  'String',
  'Vec',
  'Option',
  'Result',
  'Box',
  'HashMap',
  'HashSet',
  'BTreeMap',
];

const MACROS = [
  'println!',
  'print!',
  'eprintln!',
  'eprint!',
  'format!',
  'vec!',
  'assert!',
  'assert_eq!',
  'assert_ne!',
  'panic!',
  'dbg!',
  'todo!',
  'unimplemented!',
  'write!',
  'writeln!',
];

// Injected runtime helpers — signatures mirror PROMPT_HELPER in
// crates/mars-runtime so completions match what cells can actually call.
const MARS_HELPERS: { label: string; info: string; snippet: string }[] = [
  {
    label: 'mars_table',
    info: 'Emit a typed table output: mars_table(&["col"], &[vec!["v"]])',
    snippet: 'mars_table(&[${1:columns}], &[${2:rows}])',
  },
  {
    label: 'mars_csv',
    info: 'Parse CSV text into a table output: mars_csv(text)',
    snippet: 'mars_csv(${1:text})',
  },
  {
    label: 'mars_display_html',
    info: 'Emit a sanitized HTML output',
    snippet: 'mars_display_html(${1:html})',
  },
  {
    label: 'mars_display_latex',
    info: 'Emit a LaTeX math output',
    snippet: 'mars_display_latex(${1:latex})',
  },
  {
    label: 'mars_display_image',
    info: 'Emit an image output: mars_display_image("image/png", &bytes)',
    snippet: 'mars_display_image(${1:"image/png"}, ${2:&bytes})',
  },
  {
    label: 'mars_display_markdown',
    info: 'Emit a Markdown output',
    snippet: 'mars_display_markdown(${1:markdown})',
  },
  {
    label: 'mars_display_mime',
    info: 'Emit a MIME-bundle output',
    snippet: 'mars_display_mime(${1:mime}, ${2:data})',
  },
  {
    label: 'mars_chart',
    info: 'Emit a Vega-Lite chart output',
    snippet: 'mars_chart(${1:spec})',
  },
  {
    label: 'mars_prompt',
    info: 'Prompt for input during execution',
    snippet: 'mars_prompt(${1:message})',
  },
];

let notebookSources: string[] = [];

export const updateCompletionSources = (sources: string[]): void => {
  notebookSources = sources;
};

type CompletionOption = {
  label: string;
  type: string;
  apply?: string;
  info?: string;
  boost?: number;
};

const keywordCompletions = (options: CompletionOption[]): Completion[] =>
  options.map((option) => ({
    label: option.label,
    type: option.type,
    boost: option.boost ?? 2,
    apply: option.apply ?? option.label,
    info: option.info ?? (option.type === 'keyword' ? 'Rust keyword' : undefined),
  }));

export const rustCompletionSource = (context: CompletionContext): CompletionResult | null => {
  const word = context.matchBefore(/[A-Za-z_][A-Za-z0-9_]*/);
  if (!word || (word.from === word.to && !context.explicit)) return null;

  const prefix = word.text.toLowerCase();
  const matches = (label: string) => label.toLowerCase().startsWith(prefix);

  const helperOptions: CompletionOption[] = MARS_HELPERS.filter((helper) =>
    matches(helper.label),
  ).map((helper) => ({
    label: helper.label,
    type: 'function',
    apply: helper.snippet,
    info: helper.info,
    boost: 3,
  }));

  const options: CompletionOption[] = [
    ...helperOptions,
    ...KEYWORDS.filter(matches).map((label) => ({ label, type: 'keyword' })),
    ...PRIMITIVES.filter(matches).map((label) => ({ label, type: 'constant' })),
    ...MACROS.filter(matches).map((label) => ({
      label,
      type: 'function',
      apply: `${label}()`,
    })),
    ...extractRustSymbols(notebookSources)
      .filter((symbol) => matches(symbol.name))
      .map((symbol) => ({
        label: symbol.name,
        type:
          symbol.kind === 'binding'
            ? 'variable'
            : symbol.kind === 'function'
              ? 'function'
              : 'class',
      })),
  ];

  if (options.length === 0) return null;
  return {
    from: word.from,
    options: keywordCompletions(options),
    filter: false,
  };
};
