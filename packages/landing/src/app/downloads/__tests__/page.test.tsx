import { render, screen } from '@testing-library/react';
import DownloadsPage from '../page';

describe('DownloadsPage', () => {
  it('exports component', () => {
    expect(typeof DownloadsPage).toBe('function');
  });

  it('renders download heading and platform cards', () => {
    render(<DownloadsPage />);
    expect(screen.getByText(/get project mars for your platform/i)).toBeInTheDocument();
    expect(screen.getByText('macOS')).toBeInTheDocument();
    expect(screen.getByText('Windows')).toBeInTheDocument();
    expect(screen.getByText('Linux')).toBeInTheDocument();
  });

  it('renders platform descriptions', () => {
    render(<DownloadsPage />);
    expect(screen.getByText(/apple silicon and intel/i)).toBeInTheDocument();
    expect(screen.getByText(/x64 installer/i)).toBeInTheDocument();
    expect(screen.getByText(/appimage and .deb packages/i)).toBeInTheDocument();
  });

  it('renders download links for each platform', () => {
    render(<DownloadsPage />);
    expect(screen.getByText(/download .dmg/i)).toBeInTheDocument();
    expect(screen.getByText(/download .msi/i)).toBeInTheDocument();
    expect(screen.getAllByText(/^download$/i).length).toBeGreaterThanOrEqual(1);
  });

  it('renders cargo install section', () => {
    render(<DownloadsPage />);
    expect(screen.getByText(/install via cargo/i)).toBeInTheDocument();
    expect(screen.getByText(/cargo install/i)).toBeInTheDocument();
  });

  it('renders GitHub link', () => {
    render(<DownloadsPage />);
    expect(screen.getAllByText(/github/i).length).toBeGreaterThanOrEqual(1);
  });
});
