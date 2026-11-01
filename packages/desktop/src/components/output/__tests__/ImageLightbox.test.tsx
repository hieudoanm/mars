import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import OutputView from '../OutputView';

const pngOutput = {
  type: 'image' as const,
  mime_type: 'image/png',
  data: 'aGk=',
};

describe('ImageOutput lightbox', () => {
  it('opens on click and closes on Escape', () => {
    render(<OutputView output={pngOutput} />);
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'View image full size' }));
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('dialog'));
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('toggles zoom without closing the lightbox', () => {
    render(<OutputView output={pngOutput} />);
    fireEvent.click(screen.getByRole('button', { name: 'View image full size' }));
    const dialog = screen.getByRole('dialog');
    const image = dialog.querySelector('img');
    expect(image?.className).toContain('object-contain');
    fireEvent.click(image as Element);
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(dialog.querySelector('img')?.className).toContain('max-w-none');
  });
});
