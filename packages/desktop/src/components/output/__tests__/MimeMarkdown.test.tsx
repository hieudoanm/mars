import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import OutputView from '../OutputView';

describe('MimeBundle markdown output', () => {
  it('renders text/markdown bundles as markdown', () => {
    render(
      <OutputView
        output={{
          type: 'mime_bundle',
          data: { 'text/markdown': '**bold** rendered' },
          metadata: {},
        }}
      />,
    );
    expect(screen.getByText('bold')).toBeInTheDocument();
  });
});
