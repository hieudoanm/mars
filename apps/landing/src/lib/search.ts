import type { Doc } from './docs';
import type { Post } from './blog';

export type SearchKind = 'doc' | 'post';

export type Searchable = Readonly<{
  kind: SearchKind;
  slug: string;
  title: string;
  description?: string;
  tags?: readonly string[];
  content?: string;
}>;

export type SearchResult = Searchable & Readonly<{ href: string; score: number }>;

const normalize = (value: string): string =>
  value
    .toLowerCase()
    .normalize('NFKD')
    .replace(/[^\p{L}\p{N}\s/-]+/gu, ' ')
    .replace(/\s+/g, ' ')
    .trim();

const termsOf = (query: string): string[] => normalize(query).split(' ').filter(Boolean);

export const hrefFor = (item: Pick<Searchable, 'kind' | 'slug'>): string =>
  item.kind === 'doc' ? `/docs/${item.slug}` : `/blog/${item.slug}`;

const scoreItem = (item: Searchable, terms: string[]): number => {
  const title = normalize(item.title);
  const slug = normalize(item.slug);
  const description = normalize(item.description ?? '');
  const tags = (item.tags ?? []).map(normalize);
  const content = normalize(item.content ?? '').slice(0, 4000);
  let score = 0;

  for (const term of terms) {
    let matched = false;
    if (title === term) {
      score += 12;
      matched = true;
    } else if (title.includes(term)) {
      score += 8;
      matched = true;
    }
    if (slug.includes(term)) {
      score += 5;
      matched = true;
    }
    if (tags.some((tag) => tag === term)) {
      score += 6;
      matched = true;
    } else if (tags.some((tag) => tag.includes(term))) {
      score += 4;
      matched = true;
    }
    if (description.includes(term)) {
      score += 3;
      matched = true;
    }
    if (content.includes(term)) {
      score += 1;
      matched = true;
    }
    if (!matched) return 0;
  }

  return score;
};

export const searchContent = <T extends Searchable>(
  items: readonly T[],
  query: string,
  limit = 20,
): Array<T & { href: string; score: number }> => {
  const terms = termsOf(query);
  if (terms.length === 0) return [];

  return items
    .map((item) => ({ ...item, href: hrefFor(item), score: scoreItem(item, terms) }))
    .filter((item) => item.score > 0)
    .sort((a, b) => b.score - a.score || a.title.localeCompare(b.title))
    .slice(0, Math.max(0, limit));
};

export const toSearchableDocs = (docs: readonly Doc[]): Searchable[] =>
  docs.map((doc) => ({
    kind: 'doc',
    slug: doc.slug,
    title: doc.title,
    description: doc.description,
    content: doc.content,
  }));

export const toSearchablePosts = (posts: readonly Post[]): Searchable[] =>
  posts.map((post) => ({
    kind: 'post',
    slug: post.slug,
    title: post.title,
    description: post.description,
    tags: post.tags,
    content: post.content,
  }));
