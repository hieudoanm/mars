import { render, screen } from '@testing-library/react';
import DownloadsPage from '../page';

describe('DownloadsPage', () => {
  it('exports component', () => {
    expect(typeof DownloadsPage).toBe('function');
  });

  it('renders download heading and platform cards', () => {
    render(<DownloadsPage />);
    expect(screen.getByText(/get project mars for your platform/i)).toBeInTheDocument();
    expect(screen.getAllByText('macOS').length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText('Windows').length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText('Linux').length).toBeGreaterThanOrEqual(1);
  });

  it('renders direct release download links', () => {
    render(<DownloadsPage />);
    const dmg = screen.getByText(/apple silicon \(.dmg\)/i);
    expect(dmg.closest('a')).toHaveAttribute(
      'href',
      expect.stringContaining('desktop-mars-aarch64-apple-darwin.dmg'),
    );
    const msi = screen.getByText(/msi installer/i);
    expect(msi.closest('a')).toHaveAttribute(
      'href',
      expect.stringContaining('desktop-mars-x86_64-pc-windows-msvc.msi'),
    );
    const deb = screen.getByText(/^\.deb$/i);
    expect(deb.closest('a')).toHaveAttribute(
      'href',
      expect.stringContaining('desktop-mars-x86_64-unknown-linux-gnu.deb'),
    );
  });

  it('renders CLI binary downloads', () => {
    render(<DownloadsPage />);
    expect(screen.getByText('Linux x86_64')).toBeInTheDocument();
    expect(screen.getByText('Windows x64')).toBeInTheDocument();
    const cliLinks = screen
      .getAllByRole('link')
      .filter((a) => a.getAttribute('href')?.includes('cli-mars-'));
    expect(cliLinks).toHaveLength(4);
  });

  it('does not link to extension marketplaces', () => {
    render(<DownloadsPage />);
    expect(screen.queryByText(/marketplace/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/zed\.dev\/extensions/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/plugins\.jetbrains\.com/i)).not.toBeInTheDocument();
  });

  it('warns about the unsigned macOS build', () => {
    render(<DownloadsPage />);
    expect(screen.getByText(/unsigned app workaround/i)).toBeInTheDocument();
    expect(screen.getByText(/xattr -dr com\.apple\.quarantine/)).toBeInTheDocument();
  });

  it('renders install script and cargo commands', () => {
    render(<DownloadsPage />);
    expect(screen.getByText(/install via script or cargo/i)).toBeInTheDocument();
    expect(screen.getByText(/cargo install/i)).toBeInTheDocument();
    expect(screen.getByText(/install\.sh/)).toBeInTheDocument();
  });
});
