import { describe, expect, it } from 'vitest';
import {
  hrefFor,
  searchContent,
  toSearchableDocs,
  toSearchablePosts,
  type Searchable,
} from '../search';
import { getDocs } from '../docs';
import { getPosts } from '../blog';

const item = (overrides: Partial<Searchable> = {}): Searchable => ({
  kind: 'doc',
  slug: 'test-doc',
  title: 'Test Document',
  description: 'A document about testing',
  tags: ['testing'],
  content: 'Some searchable content about kernels and notebooks.',
  ...overrides,
});

describe('hrefFor', () => {
  it('maps docs and posts to their routes', () => {
    expect(hrefFor({ kind: 'doc', slug: 'cli' })).toBe('/docs/cli');
    expect(hrefFor({ kind: 'post', slug: 'kernels' })).toBe('/blog/kernels');
  });
});

describe('searchContent', () => {
  it('returns an empty array for blank queries', () => {
    expect(searchContent([item()], '   ')).toEqual([]);
  });

  it('returns an empty array when nothing matches', () => {
    expect(searchContent([item()], 'zzzz-no-match')).toEqual([]);
  });

  it('matches on title and ranks it highest', () => {
    const exact = item({ title: 'kernels' });
    const partial = item({ slug: 'other', title: 'About kernels' });
    const results = searchContent([partial, exact], 'kernels');
    expect(results[0].title).toBe('kernels');
    expect(results[0].score).toBeGreaterThan(results[1].score);
  });

  it('matches slug, tags, description, and content', () => {
    const bySlug = searchContent([item({ title: 'X', slug: 'kernel-guide' })], 'kernel');
    expect(bySlug.length).toBe(1);

    const byTag = searchContent([item({ title: 'X', tags: ['kernels'] })], 'kernel');
    expect(byTag.length).toBe(1);

    const byDescription = searchContent(
      [item({ title: 'X', description: 'All about kernels' })],
      'kernel',
    );
    expect(byDescription.length).toBe(1);

    const byContent = searchContent([item({ title: 'X', content: 'kernel deep dive' })], 'kernel');
    expect(byContent.length).toBe(1);
  });

  it('requires every term to match', () => {
    const results = searchContent([item()], 'kernel nonexistent-term');
    expect(results).toEqual([]);
  });

  it('sorts by score then title and respects the limit', () => {
    const items = Array.from({ length: 5 }, (_, i) =>
      item({ slug: `doc-${i}`, title: `Kernel ${i}`, content: 'kernel' }),
    );
    const results = searchContent(items, 'kernel', 3);
    expect(results).toHaveLength(3);
    expect(results[0].score).toBeGreaterThanOrEqual(results[1].score);
  });

  it('handles negative limits by returning nothing', () => {
    expect(searchContent([item()], 'kernel', -1)).toEqual([]);
  });

  it('normalizes accents and punctuation in queries', () => {
    const results = searchContent([item({ title: 'Café notebook' })], 'cafe notebook');
    expect(results).toHaveLength(1);
  });

  it('works against real docs and posts', () => {
    const docs = searchContent(toSearchableDocs(getDocs()), 'kernel');
    const posts = searchContent(toSearchablePosts(getPosts()), 'rust');
    expect(docs.length).toBeGreaterThan(0);
    expect(posts.length).toBeGreaterThan(0);
    expect(docs.every((r) => r.href.startsWith('/docs/'))).toBe(true);
    expect(posts.every((r) => r.href.startsWith('/blog/'))).toBe(true);
  });
});
