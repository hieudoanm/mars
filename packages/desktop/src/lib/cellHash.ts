/**
 * Stable hash of a code cell's source, matching
 * `mars_runtime::expression::cell_source_hash` (djb2 over UTF-8 bytes).
 * Runtime panic annotations embed this hash so errors can be mapped back
 * to the originating cell regardless of execution order.
 */
export const cellSourceHash = (source: string): number => {
  const bytes = new TextEncoder().encode(source);
  let hash = 5381;
  for (const byte of bytes) {
    hash = (Math.imul(hash, 33) + byte) >>> 0;
  }
  return hash;
};
