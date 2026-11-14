import type { Metadata } from 'next';
import Link from 'next/link';
import BlogList from '@/components/blog/BlogList';
import { getPosts } from '@/lib/blog';
import { siteName, siteUrl } from '@/lib/site';

export const metadata: Metadata = {
  title: `Blog | ${siteName}`,
  description: 'Essays on Rust, notebooks, data science, machine learning, and the case for Mars. Explore insights, tutorials, and best practices.',
  keywords: 'Rust, notebooks, data science, machine learning, Mars, programming, tutorials, best practices',
  alternates: {
    canonical: `${siteUrl}/blog`,
  },
  openGraph: {
    title: `Blog | ${siteName}`,
    description: 'Essays on Rust, notebooks, data science, machine learning, and the case for Mars.',
    url: `${siteUrl}/blog`,
    siteName,
    images: [
      {
        url: `${siteUrl}/og-image.png`,
        width: 1200,
        height: 630,
        alt: 'Mars Blog',
      },
    ],
    type: 'website',
  },
  twitter: {
    card: 'summary_large_image',
    title: `Blog | ${siteName}`,
    description: 'Essays on Rust, notebooks, data science, machine learning, and the case for Mars.',
    images: [`${siteUrl}/og-image.png`],
  },
};

export default function BlogPage() {
  const posts = getPosts();

  const blogSchema = {
    '@context': 'https://schema.org',
    '@type': 'Blog',
    name: `${siteName} Blog`,
    description: 'Essays on Rust, notebooks, data science, machine learning, and the case for Mars.',
    url: `${siteUrl}/blog`,
    publisher: {
      '@type': 'Organization',
      name: siteName,
      url: siteUrl,
    },
    blogPost: posts.slice(0, 10).map((post) => ({
      '@type': 'BlogPosting',
      headline: post.title,
      description: post.description,
      url: `${siteUrl}/blog/${post.slug}`,
      datePublished: post.date,
      author: {
        '@type': 'Person',
        name: post.author || 'Mars Team',
      },
    })),
  };

  return (
    <div>
      <script
        type='application/ld+json'
        dangerouslySetInnerHTML={{ __html: JSON.stringify(blogSchema) }}
      />
      <div className='mb-10'>
        <h1 className='text-ink text-3xl font-bold tracking-tight'>Blog</h1>
        <p className='text-muted mt-2'>
          Essays on Rust, notebooks, data science, and the case for Mars.
        </p>
        <p className='text-muted mt-1 text-sm'>
          {posts.length} {posts.length === 1 ? 'post' : 'posts'}
        </p>
        <Link
          href='/blog/series'
          className='text-orange-2 hover:text-orange-1 mt-4 inline-flex items-center gap-2 text-sm font-medium transition'>
          Browse series →
        </Link>
      </div>

      <BlogList posts={posts} />
    </div>
  );
}
