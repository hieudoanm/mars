import { render, screen } from '@testing-library/react';

const { getDocsMock } = vi.hoisted(() => ({ getDocsMock: vi.fn() }));

vi.mock('@/lib/docs', () => ({ getDocs: getDocsMock }));

import DocsLayout, { metadata } from '../layout';

describe('docs layout', () => {
  beforeEach(() => {
    getDocsMock.mockReturnValue([
      { slug: 'getting-started', title: 'Getting Started' },
      { slug: 'runtime', title: 'Runtime' },
    ]);
  });

  it('renders the shared navigation, document links, and page content', () => {
    render(<DocsLayout><h1>Guide content</h1></DocsLayout>);

    expect(screen.getByRole('link', { name: 'Mars home' })).toHaveAttribute('href', '/');
    expect(screen.getByRole('navigation', { name: 'Primary navigation' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Getting Started' })).toHaveAttribute(
      'href',
      '/docs/getting-started',
    );
    expect(screen.getByRole('link', { name: 'Runtime' })).toHaveAttribute('href', '/docs/runtime');
    expect(screen.getByRole('link', { name: /repository docs/i })).toHaveAttribute(
      'href',
      'https://github.com/hieudoanm/mars/tree/main/docs',
    );
    expect(screen.getByRole('heading', { name: 'Guide content' })).toBeInTheDocument();
    expect(getDocsMock).toHaveBeenCalledOnce();
  });

  it('exports documentation metadata', () => {
    expect(metadata).toMatchObject({
      title: 'Documentation',
      description: expect.stringContaining('Guides for installing Mars'),
    });
  });
});
