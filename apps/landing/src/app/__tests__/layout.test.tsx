import { render } from '@testing-library/react';
import { renderToStaticMarkup } from 'react-dom/server';
import RootLayout from '../layout';

describe('RootLayout', () => {
  it('renders children inside html and body', () => {
    const markup = renderToStaticMarkup(<RootLayout>Hello Mars</RootLayout>);
    expect(markup).toContain('<html lang="en"><head></head><body>Hello Mars</body></html>');
  });

  it('sets lang attribute to en', () => {
    render(<RootLayout>content</RootLayout>);
    expect(document.documentElement.lang).toBe('en');
  });
});
