import { repositoryUrl } from './site';

export const releaseDownloadUrl = (asset: string): string =>
  `${repositoryUrl}/releases/latest/download/${asset}`;

export const releasePageUrl = `${repositoryUrl}/releases/latest`;

export interface DownloadLink {
  label: string;
  href: string;
}

export interface DownloadItem {
  platform: string;
  description: string;
  links: DownloadLink[];
}

export const cliDownloads: DownloadItem[] = [
  {
    platform: 'Linux x86_64',
    description: 'Prebuilt binary',
    links: [
      {
        label: 'Download',
        href: releaseDownloadUrl('cli-mars-x86_64-unknown-linux-gnu'),
      },
    ],
  },
  {
    platform: 'macOS',
    description: 'Apple Silicon and Intel',
    links: [
      {
        label: 'Apple Silicon',
        href: releaseDownloadUrl('cli-mars-aarch64-apple-darwin'),
      },
      {
        label: 'Intel',
        href: releaseDownloadUrl('cli-mars-x86_64-apple-darwin'),
      },
    ],
  },
  {
    platform: 'Windows x64',
    description: 'Prebuilt executable',
    links: [
      {
        label: 'Download .exe',
        href: releaseDownloadUrl('cli-mars-x86_64-pc-windows-msvc.exe'),
      },
    ],
  },
];

export const desktopDownloads: DownloadItem[] = [
  {
    platform: 'macOS',
    description: 'Apple Silicon and Intel builds',
    links: [
      {
        label: 'Apple Silicon (.dmg)',
        href: releaseDownloadUrl('desktop-mars-aarch64-apple-darwin.dmg'),
      },
      {
        label: 'Intel (.dmg)',
        href: releaseDownloadUrl('desktop-mars-x86_64-apple-darwin.dmg'),
      },
    ],
  },
  {
    platform: 'Windows',
    description: 'x64 installers',
    links: [
      {
        label: 'MSI installer',
        href: releaseDownloadUrl('desktop-mars-x86_64-pc-windows-msvc.msi'),
      },
      {
        label: 'NSIS setup (.exe)',
        href: releaseDownloadUrl('desktop-mars-x86_64-pc-windows-msvc-setup.exe'),
      },
    ],
  },
  {
    platform: 'Linux',
    description: 'AppImage and package formats',
    links: [
      {
        label: '.deb',
        href: releaseDownloadUrl('desktop-mars-x86_64-unknown-linux-gnu.deb'),
      },
      {
        label: '.rpm',
        href: releaseDownloadUrl('desktop-mars-x86_64-unknown-linux-gnu.rpm'),
      },
      {
        label: 'AppImage',
        href: releaseDownloadUrl('desktop-mars-x86_64-unknown-linux-gnu.AppImage'),
      },
    ],
  },
];

export const extensionDownloads: DownloadItem[] = [
  {
    platform: 'VS Code',
    description: 'Full notebook editor with execution and rich output',
    links: [
      {
        label: 'Direct .vsix',
        href: releaseDownloadUrl('extension-vscode-mars.vsix'),
      },
      {
        label: 'Source on GitHub',
        href: `${repositoryUrl}/tree/master/extensions/vscode`,
      },
    ],
  },
  {
    platform: 'Zed',
    description: 'Syntax highlighting, grammar support, and MCP integration',
    links: [
      {
        label: 'Direct .wasm',
        href: releaseDownloadUrl('extension-zed-mars.wasm'),
      },
      {
        label: 'Source on GitHub',
        href: `${repositoryUrl}/tree/master/extensions/zed`,
      },
    ],
  },
  {
    platform: 'JetBrains',
    description: 'File type, cell execution, and kernel tool window',
    links: [
      { label: 'Plugin zip', href: releasePageUrl },
      {
        label: 'Source on GitHub',
        href: `${repositoryUrl}/tree/master/extensions/jetbrains`,
      },
    ],
  },
];
