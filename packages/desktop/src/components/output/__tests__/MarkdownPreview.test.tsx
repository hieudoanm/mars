import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import MarkdownPreview from '../MarkdownPreview';

describe('MarkdownPreview', () => {
  it('renders GFM alerts as callouts', () => {
    render(<MarkdownPreview source={'> [!WARNING]\n> Be careful'} />);
    expect(screen.getByText('WARNING')).toBeInTheDocument();
    expect(screen.getByText(/Be careful/)).toBeInTheDocument();
  });

  it('renders plain blockquotes without alert styling', () => {
    render(<MarkdownPreview source={'> just a quote'} />);
    expect(screen.queryByText('NOTE')).not.toBeInTheDocument();
    expect(screen.getByText(/just a quote/)).toBeInTheDocument();
  });

  it('resolves attachment references to data URIs', () => {
    const png = 'data:image/png;base64,iVBORw0KGgo=';
    const { container } = render(
      <MarkdownPreview
        source={'![pic](attachment:image-1.png)'}
        attachments={{ 'image-1.png': png }}
      />,
    );
    const img = container.querySelector('img');
    expect(img).toHaveAttribute('src', png);
  });
});
