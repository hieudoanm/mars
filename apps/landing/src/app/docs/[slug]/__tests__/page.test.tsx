import { render, screen } from '@testing-library/react';

const { mockDocs, notFoundMock } = vi.hoisted(() => ({
  mockDocs: [
  {
    slug: 'getting-started',
    title: 'Getting Started',
    content: '# Getting Started\n\nWelcome to Mars.',
    description: 'Mars getting-started documentation',
    order: 1,
    author: 'Mars Team',
  },
  {
    slug: 'cli',
    title: 'CLI',
    content: '# CLI reference\n\nThe mars CLI.',
    description: 'Mars cli documentation',
    order: 3,
    author: 'Mars Team',
  },
  ],
  notFoundMock: vi.fn(() => {
    throw new Error('NEXT_NOT_FOUND');
  }),
}));

vi.mock('@/lib/docs', () => ({
  getDocs: () => mockDocs,
  getDoc: (slug: string) => mockDocs.find((d) => d.slug === slug),
}));

vi.mock('next/navigation', () => ({
  notFound: notFoundMock,
}));

import DocPage, { generateMetadata, generateStaticParams } from '../page';

describe('DocPage', () => {
  beforeEach(() => {
    notFoundMock.mockClear();
  });

  it('generates static params for every documentation page', () => {
    expect(generateStaticParams()).toEqual([
      { slug: 'getting-started' },
      { slug: 'cli' },
    ]);
  });

  it('generates metadata from the matching documentation page', async () => {
    await expect(
      generateMetadata({ params: Promise.resolve({ slug: 'cli' }) }),
    ).resolves.toEqual({ title: 'CLI | Mars' });
  });

  it('returns empty metadata when a page does not exist', async () => {
    await expect(
      generateMetadata({ params: Promise.resolve({ slug: 'missing' }) }),
    ).resolves.toEqual({});
  });

  it('renders doc content and navigation', async () => {
    render(await DocPage({ params: Promise.resolve({ slug: 'getting-started' }) }));
    expect(screen.getByText('Welcome to Mars.')).toBeInTheDocument();
    expect(screen.getByText('→')).toBeInTheDocument();
    expect(screen.getByText('CLI')).toBeInTheDocument();
  });

  it('renders next link when available', async () => {
    render(await DocPage({ params: Promise.resolve({ slug: 'getting-started' }) }));
    expect(screen.getByText('Next')).toBeInTheDocument();
    expect(screen.getByText('CLI')).toBeInTheDocument();
  });

  it('does not render previous link for first doc', async () => {
    render(await DocPage({ params: Promise.resolve({ slug: 'getting-started' }) }));
    expect(screen.getByText('Welcome to Mars.')).toBeInTheDocument();
    expect(screen.queryByText('Previous')).toBeNull();
  });

  it('does not render next link for last doc', async () => {
    render(await DocPage({ params: Promise.resolve({ slug: 'cli' }) }));
    expect(screen.getByRole('heading', { name: 'CLI reference' })).toBeInTheDocument();
    expect(screen.queryByText('Next')).toBeNull();
  });

  it('calls notFound for missing doc', async () => {
    await expect(
      DocPage({ params: Promise.resolve({ slug: 'missing' }) }),
    ).rejects.toThrow('NEXT_NOT_FOUND');
    expect(notFoundMock).toHaveBeenCalled();
  });
});
