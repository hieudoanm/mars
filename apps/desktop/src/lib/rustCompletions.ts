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

let notebookSources: string[] = [];

export const updateCompletionSources = (sources: string[]): void => {
  notebookSources = sources;
};

const keywordCompletions = (options: { label: string; type: string }[]): Completion[] =>
  options.map(({ label, type }) => ({
    label,
    type,
    boost: 2,
    apply: label,
    info: type === 'keyword' ? 'Rust keyword' : undefined,
  }));

export const rustCompletionSource = (context: CompletionContext): CompletionResult | null => {
  const word = context.matchBefore(/[A-Za-z_][A-Za-z0-9_]*/);
  if (!word || (word.from === word.to && !context.explicit)) return null;

  const prefix = word.text.toLowerCase();
  const matches = (label: string) => label.toLowerCase().startsWith(prefix);

  const options: { label: string; type: string }[] = [
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
