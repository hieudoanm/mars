import { fireEvent, render, screen } from '@testing-library/react';
import type { Post } from '@/lib/blog';
import BlogSearch from '../BlogSearch';

const posts: Post[] = [
  {
    slug: 'why-rust',
    title: 'Why Rust?',
    description: 'Rust is fast and safe.',
    author: 'Mars Team',
    date: '2026-11-06',
    order: 1,
    tags: ['rust', 'systems'],
    content: '# Why Rust?\n\nRust has ownership and borrowing.',
  },
  {
    slug: 'data-science-tools',
    title: '10 Essential Data Science Tools',
    description: 'Python and Rust data science tools.',
    author: 'Mars Team',
    date: '2026-11-06',
    order: 3,
    tags: ['data-science', 'rust', 'python'],
    content: '# Data science\n\nNumPy maps to ndarray and pandas maps to polars.',
  },
  {
    slug: 'machine-learning-libraries',
    title: '10 Machine Learning Libraries',
    description: 'Machine learning libraries in Rust.',
    author: 'Mars Team',
    date: '2026-11-06',
    order: 4,
    tags: ['machine-learning', 'rust'],
    content: '# ML\n\nscikit-learn maps to linfa.',
  },
];

describe('BlogSearch', () => {
  it('renders search input and posts', () => {
    render(<BlogSearch posts={posts} />);
    expect(screen.getByRole('textbox', { name: /search posts/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /why rust\?/i })).toHaveAttribute(
      'href',
      '/blog/why-rust',
    );
    expect(screen.getByRole('link', { name: /10 essential data science tools/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /10 machine learning libraries/i })).toBeInTheDocument();
  });

  it('renders tags from posts', () => {
    render(<BlogSearch posts={posts} />);
    expect(screen.getByRole('button', { name: 'All' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'rust' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'data-science' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'machine-learning' })).toBeInTheDocument();
    expect(screen.getAllByText('rust').length).toBeGreaterThan(0);
  });

  it('filters posts by query text', () => {
    render(<BlogSearch posts={posts} />);
    fireEvent.change(screen.getByRole('textbox', { name: /search posts/i }), {
      target: { value: 'polars' },
    });
    expect(screen.getByRole('link', { name: /10 essential data science tools/i })).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: /why rust\?/i })).toBeNull();
    expect(screen.queryByRole('link', { name: /10 machine learning libraries/i })).toBeNull();
  });

  it('filters posts by tag', () => {
    render(<BlogSearch posts={posts} />);
    fireEvent.click(screen.getByRole('button', { name: 'machine-learning' }));
    expect(screen.getByRole('link', { name: /10 machine learning libraries/i })).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: /why rust\?/i })).toBeNull();
    expect(screen.queryByRole('link', { name: /10 essential data science tools/i })).toBeNull();
  });

  it('combines tag filter and query', () => {
    render(<BlogSearch posts={posts} />);
    fireEvent.click(screen.getByRole('button', { name: 'rust' }));
    fireEvent.change(screen.getByRole('textbox', { name: /search posts/i }), {
      target: { value: 'polars' },
    });
    expect(screen.getByRole('link', { name: /10 essential data science tools/i })).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: /why rust\?/i })).toBeNull();
  });

  it('clears tag filter when All is clicked', () => {
    render(<BlogSearch posts={posts} />);
    fireEvent.click(screen.getByRole('button', { name: 'machine-learning' }));
    expect(screen.queryByRole('link', { name: /why rust\?/i })).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'All' }));
    expect(screen.getByRole('link', { name: /why rust\?/i })).toBeInTheDocument();
  });

  it('shows empty state when nothing matches', () => {
    render(<BlogSearch posts={posts} />);
    fireEvent.change(screen.getByRole('textbox', { name: /search posts/i }), {
      target: { value: 'zzzz-no-match' },
    });
    expect(screen.getByText(/no matching posts/i)).toBeInTheDocument();
  });
});
