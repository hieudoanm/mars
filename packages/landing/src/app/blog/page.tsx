import type { FC } from 'react';
import type { NextPage } from 'next';
import BlogSearch from '@/components/blog/BlogSearch';
import { getPosts } from '@/lib/blog';

export const dynamicParams = false;

export const generateStaticParams = () => {
  return getPosts().map((post) => ({ slug: post.slug }));
};

const BlogPage: FC & NextPage = () => {
  const posts = getPosts();

  return (
    <div>
      <div className='mb-10'>
        <h1 className='text-ink text-3xl font-bold tracking-tight'>Blog</h1>
        <p className='text-muted mt-2'>
          Essays on Rust, notebooks, data science, and the case for Mars.
        </p>
      </div>

      {posts.length === 0 ? (
        <p className='text-muted'>No posts yet.</p>
      ) : (
        <BlogSearch posts={posts} />
      )}
    </div>
  );
};

export default BlogPage;