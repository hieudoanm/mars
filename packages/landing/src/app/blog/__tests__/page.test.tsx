import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import BlogPage, { metadata } from '../page';
import { getPosts } from '@/lib/blog';

describe('BlogPage', () => {
  it('renders the blog heading and post count', () => {
    render(<BlogPage />);
    expect(screen.getByRole('heading', { name: 'Blog' })).toBeInTheDocument();
    const count = getPosts().length;
    expect(screen.getByText(`${count} ${count === 1 ? 'post' : 'posts'}`)).toBeInTheDocument();
  });

  it('links to the series index', () => {
    render(<BlogPage />);
    expect(screen.getByRole('link', { name: /browse series/i })).toHaveAttribute(
      'href',
      '/blog/series',
    );
  });

  it('renders structured data scripts', () => {
    const { container } = render(<BlogPage />);
    const scripts = container.querySelectorAll('script[type="application/ld+json"]');
    expect(scripts.length).toBeGreaterThanOrEqual(2);
  });
});

describe('blog metadata', () => {
  it('has title, description, and canonical URL', () => {
    expect(metadata.title).toContain('Blog');
    expect(metadata.description).toBeTruthy();
    expect(metadata.alternates?.canonical).toMatch(/\/blog$/);
  });
});
