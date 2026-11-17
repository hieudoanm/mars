import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import BlogList from '../BlogList';
import type { Post } from '@/lib/blog';

const makePost = (index: number): Post => ({
  slug: `post-${index}`,
  title: `Post ${index}`,
  description: `Description ${index}`,
  author: 'Test Author',
  date: '2026-01-01',
  tags: index % 2 === 0 ? ['rust', 'notebooks'] : [],
  content: '# Content',
});

describe('BlogList', () => {
  it('shows an empty state when there are no posts', () => {
    render(<BlogList posts={[]} />);
    expect(screen.getByText('No posts yet.')).toBeInTheDocument();
  });

  it('renders post cards with metadata, tags, and links', () => {
    render(<BlogList posts={[makePost(2)]} />);
    const link = screen.getByRole('link', { name: /post 2/i });
    expect(link).toHaveAttribute('href', '/blog/post-2');
    expect(screen.getByText('2026-01-01')).toBeInTheDocument();
    expect(screen.getByText('· Test Author')).toBeInTheDocument();
    expect(screen.getByText('rust')).toBeInTheDocument();
    expect(screen.getByText('Read post →')).toBeInTheDocument();
  });

  it('omits optional fields when absent', () => {
    const post = { ...makePost(2), description: undefined, author: undefined, date: undefined };
    render(<BlogList posts={[post]} />);
    expect(screen.queryByText(/Test Author/)).not.toBeInTheDocument();
  });

  it('shows pagination controls beyond one page', () => {
    const posts = Array.from({ length: 12 }, (_, i) => makePost(i));
    render(<BlogList posts={posts} />);
    expect(screen.getByText('Page 1 of 2')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /previous/i })).toBeDisabled();

    fireEvent.click(screen.getByRole('button', { name: /next/i }));
    expect(screen.getByText('Page 2 of 2')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /next/i })).toBeDisabled();
    expect(screen.getByRole('link', { name: /post 11/i })).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: /previous/i }));
    expect(screen.getByText('Page 1 of 2')).toBeInTheDocument();
  });
});
