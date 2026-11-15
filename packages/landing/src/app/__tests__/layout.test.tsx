import { render, screen } from '@testing-library/react';
import RootLayout from '../layout';

describe('RootLayout', () => {
  it('renders children inside body', () => {
    render(<RootLayout>Hello Project Mars</RootLayout>);
    expect(screen.getByText('Hello Project Mars')).toBeInTheDocument();
  });

  it('sets lang attribute to en', () => {
    render(<RootLayout>content</RootLayout>);
    expect(document.documentElement.lang).toBe('en');
  });
});
