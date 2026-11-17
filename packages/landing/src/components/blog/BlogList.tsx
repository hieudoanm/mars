'use client';

import { useState } from 'react';
import Link from 'next/link';
import type { Post } from '@/lib/blog';

const POSTS_PER_PAGE = 10;

type BlogListProps = Readonly<{ posts: Post[] }>;

export default function BlogList({ posts }: BlogListProps) {
  const [currentPage, setCurrentPage] = useState(1);
  const totalPages = Math.ceil(posts.length / POSTS_PER_PAGE);
  const startIndex = (currentPage - 1) * POSTS_PER_PAGE;
  const endIndex = startIndex + POSTS_PER_PAGE;
  const paginatedPosts = posts.slice(startIndex, endIndex);

  if (posts.length === 0) {
    return <p className='text-muted'>No posts yet.</p>;
  }

  return (
    <>
      <ul className='grid gap-4 sm:grid-cols-2'>
        {paginatedPosts.map((post) => (
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

      {totalPages > 1 && (
        <div className='mt-8 flex items-center justify-center gap-2'>
          <button
            onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
            disabled={currentPage === 1}
            className='border-line bg-panel hover:border-orange/60 hover:bg-panel-2 rounded-lg border px-4 py-2 text-sm transition disabled:cursor-not-allowed disabled:opacity-50'>
            ← Previous
          </button>
          <span className='text-muted mx-4 text-sm'>
            Page {currentPage} of {totalPages}
          </span>
          <button
            onClick={() => setCurrentPage((p) => Math.min(totalPages, p + 1))}
            disabled={currentPage === totalPages}
            className='border-line bg-panel hover:border-orange/60 hover:bg-panel-2 rounded-lg border px-4 py-2 text-sm transition disabled:cursor-not-allowed disabled:opacity-50'>
            Next →
          </button>
        </div>
      )}
    </>
  );
}
