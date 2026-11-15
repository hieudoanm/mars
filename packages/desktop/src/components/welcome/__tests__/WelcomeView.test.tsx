import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import WelcomeView from '../WelcomeView';

const setup = (recents: string[] = []) => {
  const handlers = {
    onOpenFile: vi.fn(),
    onOpenFolder: vi.fn(),
    onNewNotebook: vi.fn(),
    onNewTemplate: vi.fn(),
    onOpenRecent: vi.fn(),
  };
  render(<WelcomeView recents={recents} {...handlers} />);
  return handlers;
};

describe('WelcomeView', () => {
  it('renders the brand, tagline, and primary actions', () => {
    setup();
    expect(screen.getByText('Project Mars')).toBeInTheDocument();
    expect(
      screen.getByText('Explore ideas. Execute code. Discover possibilities.'),
    ).toBeInTheDocument();
    for (const name of ['Open notebook…', 'Open folder…', 'New notebook', 'From template…']) {
      expect(screen.getByRole('button', { name })).toBeInTheDocument();
    }
  });

  it('routes each action to its handler', () => {
    const handlers = setup(['/notes/analysis.mars']);
    fireEvent.click(screen.getByRole('button', { name: 'Open notebook…' }));
    fireEvent.click(screen.getByRole('button', { name: 'Open folder…' }));
    fireEvent.click(screen.getByRole('button', { name: 'New notebook' }));
    fireEvent.click(screen.getByRole('button', { name: 'From template…' }));
    fireEvent.click(screen.getByRole('button', { name: /analysis\.mars/ }));
    expect(handlers.onOpenFile).toHaveBeenCalledOnce();
    expect(handlers.onOpenFolder).toHaveBeenCalledOnce();
    expect(handlers.onNewNotebook).toHaveBeenCalledOnce();
    expect(handlers.onNewTemplate).toHaveBeenCalledOnce();
    expect(handlers.onOpenRecent).toHaveBeenCalledWith('/notes/analysis.mars');
  });

  it('hides the recent list when empty', () => {
    setup();
    expect(screen.queryByText('Recent')).not.toBeInTheDocument();
  });
});
