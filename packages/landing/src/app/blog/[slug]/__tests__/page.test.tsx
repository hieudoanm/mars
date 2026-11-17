import { render, act } from '@testing-library/react';
import { vi } from 'vitest';

const notFoundMock = vi.hoisted(() =>
  vi.fn(() => {
    throw new Error('NEXT_NOT_FOUND');
  }),
);

vi.mock('next/navigation', () => ({
  notFound: notFoundMock,
}));

import BlogPostPage, { generateMetadata, generateStaticParams } from '../page';
import { getPost, getPosts } from '@/lib/blog';

describe('BlogPostPage', () => {
  beforeEach(() => {
    notFoundMock.mockClear();
  });

  it('renders post content for a valid slug', async () => {
    let result: ReturnType<typeof render> | null = null;
    await act(async () => {
      result = render(BlogPostPage({ params: Promise.resolve({ slug: 'kernels' }) }));
    });
    expect(result!.container).not.toBeEmptyDOMElement();
    expect(result!.container.textContent).toContain(
      'Jupyter Kernels: Language Support Beyond Python',
    );
  });

  it('renders previous and next post navigation', async () => {
    const posts = getPosts();
    const middle = posts[1];
    let result: ReturnType<typeof render> | null = null;
    await act(async () => {
      result = render(BlogPostPage({ params: Promise.resolve({ slug: middle.slug }) }));
    });
    expect(result!.getByText('Previous')).toBeInTheDocument();
    expect(result!.getByText('Next')).toBeInTheDocument();
  });

  it('omits previous link for the first post', async () => {
    const first = getPosts()[0];
    let result: ReturnType<typeof render> | null = null;
    await act(async () => {
      result = render(BlogPostPage({ params: Promise.resolve({ slug: first.slug }) }));
    });
    expect(result!.queryByText('Previous')).toBeNull();
  });

  it('calls notFound for a missing slug', async () => {
    await expect(
      act(async () => {
        render(BlogPostPage({ params: Promise.resolve({ slug: 'missing-post' }) }));
      }),
    ).rejects.toThrow('NEXT_NOT_FOUND');
    expect(notFoundMock).toHaveBeenCalled();
  });
});

describe('generateStaticParams', () => {
  it('returns a param for every post', () => {
    const params = generateStaticParams();
    expect(params).toHaveLength(getPosts().length);
    expect(params.every((p) => typeof p.slug === 'string')).toBe(true);
  });
});

describe('generateMetadata', () => {
  it('builds article metadata for a valid slug', async () => {
    const post = getPost('kernels')!;
    const metadata = await generateMetadata({
      params: Promise.resolve({ slug: 'kernels' }),
    });
    expect(metadata.title).toContain(post.title);
    expect(metadata.alternates?.canonical).toContain('/blog/kernels');
    expect(metadata.openGraph?.url).toContain('/blog/kernels');
    expect(metadata.openGraph?.title).toContain(post.title);
  });

  it('returns empty metadata for a missing slug', async () => {
    const metadata = await generateMetadata({
      params: Promise.resolve({ slug: 'missing-post' }),
    });
    expect(metadata).toEqual({});
  });
});
