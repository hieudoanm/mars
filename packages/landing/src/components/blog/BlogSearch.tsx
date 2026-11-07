'use client';

import Link from 'next/link';
import { useMemo, useState, type FC } from 'react';
import type { Post } from '@/lib/blog';
import { searchContent, toSearchablePosts } from '@/lib/search';

const BlogSearch: FC<Readonly<{ posts: Post[] }>> = ({ posts }) => {
  const [query, setQuery] = useState('');
  const [activeTag, setActiveTag] = useState<string>();

  const tags = useMemo(
    () =>
      Array.from(new Set(posts.flatMap((post) => post.tags))).sort((a, b) => a.localeCompare(b)),
    [posts],
  );

  const filtered = useMemo(() => {
    const tagged = activeTag ? posts.filter((post) => post.tags.includes(activeTag)) : posts;
    const trimmed = query.trim();
    if (!trimmed) return tagged;
    const results = searchContent(toSearchablePosts(tagged), trimmed, tagged.length);
    const bySlug = new Set(results.map((result) => result.slug));
    return tagged.filter((post) => bySlug.has(post.slug));
  }, [activeTag, posts, query]);

  return (
    <div>
      <div className='mb-6 flex flex-col gap-3'>
        <label className='text-2xs text-muted font-bold uppercase' htmlFor='blog-search'>
          Search posts
        </label>
        <input
          id='blog-search'
          aria-label='Search posts'
          className='border-line bg-bg focus:border-orange rounded-lg border px-3 py-2 text-sm focus:outline-none'
          placeholder='Search by title, description, tag, or content'
          value={query}
          onChange={(event) => setQuery(event.currentTarget.value)}
        />
        {tags.length > 0 ? (
          <div className='flex flex-wrap gap-2' aria-label='Post tags'>
            <button
              type='button'
              className={`rounded-full border px-2.5 py-1 text-xs transition ${activeTag ? 'border-line text-muted hover:border-orange/40 hover:text-ink' : 'border-orange/50 bg-orange/10 text-orange-2'}`}
              onClick={() => setActiveTag(undefined)}>
              All
            </button>
            {tags.map((tag) => (
              <button
                key={tag}
                type='button'
                className={`rounded-full border px-2.5 py-1 text-xs transition ${activeTag === tag ? 'border-orange/50 bg-orange/10 text-orange-2' : 'border-line text-muted hover:border-orange/40 hover:text-ink'}`}
                onClick={() => setActiveTag((current) => (current === tag ? undefined : tag))}>
                {tag}
              </button>
            ))}
          </div>
        ) : null}
      </div>

      {filtered.length === 0 ? (
        <p className='text-muted'>No matching posts.</p>
      ) : (
        <ul className='grid gap-4 sm:grid-cols-2'>
          {filtered.map((post) => (
            <li key={post.slug}>
              <Link
                href={`/blog/${post.slug}`}
                className='group border-line bg-panel hover:border-orange/60 hover:bg-panel-2 flex h-full flex-col rounded-xl border p-5 transition'>
                <div className='text-muted flex flex-wrap items-center gap-x-3 gap-y-1 text-xs'>
                  {post.date ? <time dateTime={post.date}>{post.date}</time> : null}
                  {post.author ? <span>· {post.author}</span> : null}
                </div>
                <h2 className='text-ink group-hover:text-orange-2 mt-2 text-lg font-semibold'>
                  {post.title}
                </h2>
                {post.description ? (
                  <p className='text-muted mt-1 line-clamp-3 text-sm'>{post.description}</p>
                ) : null}
                {post.tags.length > 0 ? (
                  <div className='mt-3 flex flex-wrap gap-1.5'>
                    {post.tags.map((tag) => (
                      <span
                        key={tag}
                        className='border-line bg-bg text-muted text-2xs rounded-full border px-2 py-0.5'>
                        {tag}
                      </span>
                    ))}
                  </div>
                ) : null}
                <span className='text-orange-2 mt-auto pt-4 text-sm font-medium'>Read post →</span>
              </Link>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
};

export default BlogSearch;
