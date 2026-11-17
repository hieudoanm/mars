import { describe, expect, it } from 'vitest';
import { getPost, getPosts } from '../blog';

describe('getPosts', () => {
  it('returns the posts shipped in md/posts', () => {
    const posts = getPosts();
    expect(posts.length).toBeGreaterThan(0);
  });

  it('parses frontmatter into post fields', () => {
    const post = getPost('kernels');
    expect(post).toBeDefined();
    expect(post!.title).toBe('Jupyter Kernels: Language Support Beyond Python');
    expect(post!.author).toBe('Project Mars Team');
    expect(post!.date).toBe('2026-11-14');
    expect(post!.order).toBe(45);
    expect(post!.tags).toContain('kernels');
    expect(post!.tags).toContain('rust');
    expect(post!.content).toContain('# These Jupyter Kernels');
    expect(post!.content.startsWith('---')).toBe(false);
  });

  it('sorts posts by explicit order first', () => {
    const posts = getPosts();
    const ordered = posts.filter((post) => post.order !== undefined);
    const orders = ordered.map((post) => post.order);
    const sorted = [...orders].sort((a, b) => a! - b!);
    expect(orders).toEqual(sorted);
  });

  it('assigns every post a slug, title, and tags array', () => {
    for (const post of getPosts()) {
      expect(post.slug).toMatch(/^[a-z0-9-]+$/);
      expect(post.title.length).toBeGreaterThan(0);
      expect(Array.isArray(post.tags)).toBe(true);
    }
  });
});

describe('getPost', () => {
  it('finds a post by slug', () => {
    const post = getPost('language-rust');
    expect(post).toBeDefined();
    expect(post!.slug).toBe('language-rust');
  });

  it('returns undefined for an unknown slug', () => {
    expect(getPost('does-not-exist')).toBeUndefined();
  });

  it('rejects slugs with unsafe characters', () => {
    expect(getPost('../etc/passwd')).toBeUndefined();
    expect(getPost('UPPER')).toBeUndefined();
    expect(getPost('with space')).toBeUndefined();
  });
});
