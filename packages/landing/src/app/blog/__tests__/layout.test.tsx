import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import BlogLayout, { metadata } from '../layout';

describe('BlogLayout', () => {
  it('renders navbar, children, and footer', () => {
    render(
      <BlogLayout>
        <p>child content</p>
      </BlogLayout>,
    );
    expect(screen.getByText('child content')).toBeInTheDocument();
    expect(screen.getByRole('navigation', { name: 'Primary navigation' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /project mars home/i })).toBeInTheDocument();
  });
});

describe('blog layout metadata', () => {
  it('sets title and description', () => {
    expect(metadata.title).toBe('Blog');
    expect(metadata.description).toBeTruthy();
  });
});
