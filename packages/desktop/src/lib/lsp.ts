import type { LspHoverResult } from './kernelService';

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null;

export const lspHoverText = (result: LspHoverResult | null): string | null => {
  const contents = result?.contents;
  if (!contents) return null;
  const parts = Array.isArray(contents) ? contents : [contents];
  const texts = parts
    .map((part) => (isRecord(part) && typeof part.value === 'string' ? part.value : null))
    .filter((value): value is string => value !== null && value.trim().length > 0);
  return texts.length > 0 ? texts.join('\n\n') : null;
};

const isObj = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null;

/**
 * Format a `textDocument/signatureHelp` result into display text.
 * Returns `null` when there is no active signature.
 */
export const signatureHelpText = (result: unknown): string | null => {
  if (!isObj(result)) return null;
  const signatures = result.signatures;
  if (!Array.isArray(signatures) || signatures.length === 0) return null;
  const activeIndex = typeof result.activeSignature === 'number' ? result.activeSignature : 0;
  const signature = signatures[activeIndex] ?? signatures[0];
  if (!isObj(signature) || typeof signature.label !== 'string') return null;
  const activeParameter = result.activeParameter;
  let text = signature.label;
  if (typeof activeParameter === 'number') {
    const parameters = signature.parameters;
    if (Array.isArray(parameters)) {
      const parameter = parameters[activeParameter];
      if (isObj(parameter) && typeof parameter.label === 'string') {
        text += `\n↳ ${parameter.label}`;
      }
    }
  }
  return text;
};
