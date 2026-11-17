import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import SeriesPage, { metadata } from '../page';
import { getPost } from '@/lib/blog';

describe('SeriesPage', () => {
  it('renders the series heading', () => {
    render(<SeriesPage />);
    expect(screen.getByRole('heading', { name: 'Series' })).toBeInTheDocument();
  });

  it('renders every series section', () => {
    render(<SeriesPage />);
    expect(screen.getByRole('heading', { name: 'Programming Languages' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Notebook Platforms' })).toBeInTheDocument();
  });

  it('links to posts that exist', () => {
    render(<SeriesPage />);
    const links = screen.getAllByRole('link');
    const blogLinks = links.filter((link) => link.getAttribute('href')?.startsWith('/blog/'));
    expect(blogLinks.length).toBeGreaterThan(0);
    for (const link of blogLinks) {
      const slug = link.getAttribute('href')!.replace('/blog/', '');
      expect(getPost(slug), `missing post for ${slug}`).toBeDefined();
    }
  });

  it('renders titles of listed posts', () => {
    render(<SeriesPage />);
    const rust = getPost('language-rust');
    expect(rust).toBeDefined();
    expect(screen.getAllByText(rust!.title).length).toBeGreaterThan(0);
  });
});

describe('series metadata', () => {
  it('has title and canonical URL', () => {
    expect(metadata.title).toContain('Series');
    expect(metadata.alternates?.canonical).toMatch(/\/blog\/series$/);
  });
});
