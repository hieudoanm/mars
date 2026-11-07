import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import { SidebarSection } from '../SidebarSection';

describe('SidebarSection', () => {
  beforeEach(() => window.localStorage.clear());

  it('toggles content and persists the collapsed state', () => {
    render(
      <SidebarSection id='test-section' title='Test'>
        <p>Section body</p>
      </SidebarSection>,
    );
    expect(screen.getByText('Section body')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: /Test/ }));
    expect(screen.queryByText('Section body')).not.toBeInTheDocument();
    expect(JSON.parse(window.localStorage.getItem('mars.sidebar.collapsed')!)).toContain(
      'test-section',
    );
  });

  it('restores the collapsed state from storage', () => {
    window.localStorage.setItem('mars.sidebar.collapsed', JSON.stringify(['test-section']));
    render(
      <SidebarSection id='test-section' title='Test'>
        <p>Section body</p>
      </SidebarSection>,
    );
    expect(screen.queryByText('Section body')).not.toBeInTheDocument();
  });
});
