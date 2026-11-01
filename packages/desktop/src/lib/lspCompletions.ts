import type { Completion, CompletionContext, CompletionResult } from '@codemirror/autocomplete';
import type { LspCompletionItem } from './kernelService';

export type LspCompletionFetcher = (
  line: number,
  character: number,
) => Promise<{ items?: LspCompletionItem[] } | LspCompletionItem[] | null>;

const LSP_KIND_TYPES: Record<number, string> = {
  2: 'method',
  3: 'function',
  4: 'constructor',
  5: 'property',
  6: 'variable',
  7: 'class',
  8: 'interface',
  9: 'module',
  10: 'property',
  12: 'constant',
  13: 'type',
  14: 'keyword',
  15: 'snippet',
  18: 'text',
  20: 'property',
  21: 'constant',
  22: 'class',
  23: 'event',
  24: 'operator',
  25: 'type',
};

const toCompletion = (item: LspCompletionItem): Completion => ({
  label: item.label,
  type: LSP_KIND_TYPES[item.kind ?? 1] ?? 'text',
  detail: item.detail,
  apply: item.insertText ?? item.label,
  boost: 2,
});

export const createLspCompletionSource = (
  fetcher: LspCompletionFetcher,
): ((context: CompletionContext) => Promise<CompletionResult | null>) => {
  return async (context: CompletionContext) => {
    const line = context.state.doc.lineAt(context.pos);
    const result = await fetcher(line.number - 1, context.pos - line.from).catch(() => null);
    if (!result) return null;
    const items = Array.isArray(result) ? result : (result.items ?? []);
    if (items.length === 0) return null;
    const word = context.matchBefore(/\w*/);
    const from = word ? word.from : context.pos;
    return {
      from,
      options: items.slice(0, 200).map(toCompletion),
      validFor: /^\w*$/,
    };
  };
};
