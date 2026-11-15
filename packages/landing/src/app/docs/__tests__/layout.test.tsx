import { render, screen } from '@testing-library/react';
import { fireEvent } from '@testing-library/react';
import { vi } from 'vitest';

const { getDocsMock } = vi.hoisted(() => ({ getDocsMock: vi.fn() }));

vi.mock('@/lib/docs', () => ({ getDocs: getDocsMock }));

import DocsLayout, { metadata } from '../layout';

describe('docs layout', () => {
  beforeEach(() => {
    getDocsMock.mockReturnValue([
      { slug: '01-getting-started', title: 'Getting Started', description: 'Project Mars getting-started documentation' },
      { slug: '04-runtime', title: 'Runtime', description: 'Project Mars runtime documentation' },
    ]);
  });

  it('renders the shared navigation, document links, and page content', () => {
    render(<DocsLayout><h1>Guide content</h1></DocsLayout>);

    expect(screen.getByRole('link', { name: 'Project Mars home' })).toHaveAttribute('href', '/');
    expect(screen.getByRole('navigation', { name: 'Primary navigation' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Getting Started' })).toHaveAttribute(
      'href',
      '/docs/01-getting-started',
    );
    expect(screen.getByRole('link', { name: 'Runtime' })).toHaveAttribute('href', '/docs/04-runtime');
    expect(screen.getByRole('link', { name: /repository docs/i })).toHaveAttribute(
      'href',
      'https://github.com/hieudoanm/mars/tree/main/docs',
    );
    expect(screen.getByRole('heading', { name: 'Guide content' })).toBeInTheDocument();
    expect(getDocsMock).toHaveBeenCalledOnce();
  });

  it('renders a search input in the sidebar', () => {
    render(<DocsLayout><h1>Guide content</h1></DocsLayout>);
    expect(screen.getByRole('textbox', { name: /search/i })).toBeInTheDocument();
  });

  it('filters docs by title when searching', () => {
    render(<DocsLayout><h1>Guide content</h1></DocsLayout>);
    const searchInput = screen.getByRole('textbox', { name: /search/i });
    fireEvent.change(searchInput, { target: { value: 'runtime' } });
    expect(screen.getByRole('link', { name: 'Runtime' })).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'Getting Started' })).toBeNull();
  });

  it('filters docs by description when searching', () => {
    render(<DocsLayout><h1>Guide content</h1></DocsLayout>);
    const searchInput = screen.getByRole('textbox', { name: /search/i });
    fireEvent.change(searchInput, { target: { value: 'getting-started' } });
    expect(screen.getByRole('link', { name: 'Getting Started' })).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'Runtime' })).toBeNull();
  });

  it('shows all docs when search is cleared', () => {
    render(<DocsLayout><h1>Guide content</h1></DocsLayout>);
    const searchInput = screen.getByRole('textbox', { name: /search/i });
    fireEvent.change(searchInput, { target: { value: 'runtime' } });
    expect(screen.queryByRole('link', { name: 'Getting Started' })).toBeNull();
    fireEvent.change(searchInput, { target: { value: '' } });
    expect(screen.getByRole('link', { name: 'Getting Started' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Runtime' })).toBeInTheDocument();
  });

  it('exports documentation metadata', () => {
    expect(metadata).toMatchObject({
      title: 'Documentation',
      description: expect.stringContaining('Guides for installing Project Mars'),
    });
  });
});
