import type { FC } from 'react';
import type { NextPage } from 'next';
import Footer from '@/components/layout/Footer';
import Navbar from '@/components/layout/Navbar';
import { repositoryUrl } from '@/lib/site';

const sectionShell = 'mx-auto w-full max-w-shell px-6 max-sm-nav:px-4';

import { FaApple, FaWindows, FaLinux } from 'react-icons/fa';
import { FaCode, FaTerminal, FaTools } from 'react-icons/fa';

const desktopDownloads = [
  {
    platform: 'macOS',
    icon: FaApple,
    description: 'Universal build for Apple Silicon and Intel',
    links: [{ label: 'Download .dmg', href: `${repositoryUrl}/releases` }],
  },
  {
    platform: 'Windows',
    icon: FaWindows,
    description: 'x64 installer',
    links: [{ label: 'Download .msi', href: `${repositoryUrl}/releases` }],
  },
  {
    platform: 'Linux',
    icon: FaLinux,
    description: 'AppImage and .deb packages',
    links: [{ label: 'Download', href: `${repositoryUrl}/releases` }],
  },
];

const extensionDownloads = [
  {
    platform: 'VS Code',
    icon: FaCode,
    description: 'Full notebook editor with execution and rich output',
    links: [
      { label: 'VS Code Marketplace', href: 'https://marketplace.visualstudio.com/items?itemName=mars.mars' },
      { label: 'GitHub', href: `${repositoryUrl}/tree/main/extensions/vscode` },
    ],
  },
  {
    platform: 'Zed',
    icon: FaTerminal,
    description: 'Syntax highlighting, grammar support, and MCP integration',
    links: [
      { label: 'Extension Marketplace', href: 'https://zed.dev/extensions' },
      { label: 'GitHub', href: `${repositoryUrl}/tree/main/extensions/zed` },
    ],
  },
  {
    platform: 'JetBrains',
    icon: FaTools,
    description: 'File type, cell execution, and kernel tool window',
    links: [
      { label: 'Marketplace', href: 'https://plugins.jetbrains.com' },
      { label: 'GitHub', href: `${repositoryUrl}/tree/main/extensions/jetbrains` },
    ],
  },
];

const DownloadsPage: FC & NextPage = () => {
  return (
    <div className='bg-bg-alt text-ink-2 min-h-screen'>
      <Navbar
        className='max-w-shell max-sm-nav:px-4 mx-auto flex w-full items-center justify-between px-6 py-7'
        showHome
      />

      <main>
        <section className={`${sectionShell} max-sm-nav:py-15 py-20`}>
          <div className='max-w-prose-xl mb-15'>
            <div className='text-xs-plus tracking-caps-xl text-orange-4 flex items-center gap-2.5 font-bold uppercase'>
              <span className='bg-orange shadow-glow-orange h-2 w-2 rounded-full' />
              Download
            </div>
            <h1 className='text-display font-semibold-plus tracking-tighter-plus mt-5 mb-4.5 leading-tight'>
              Get Mars for your platform
            </h1>
            <p className='text-muted-2 text-lg'>
              Download the desktop app, install extensions, or use the CLI from source.
            </p>
          </div>

          <div className='mb-12'>
            <h2 className='mb-6 text-xl font-semibold-plus tracking-tight-plus'>Desktop Application</h2>
            <div className='max-md-nav:grid-cols-1 grid grid-cols-3 gap-4'>
              {desktopDownloads.map((item) => (
              <article
                key={item.platform}
                className='border-line-2 bg-feature-card rounded-xl border p-7'>
                <div className='mb-6 text-3xl'><item.icon /></div>
                <h2 className='tracking-tight-plus mb-2 text-xl'>{item.platform}</h2>
                <p className='text-sm-plus text-muted-2 mb-6'>{item.description}</p>
                <div className='flex flex-col gap-2'>
                  {item.links.map((link) => (
                    <a
                      key={link.label}
                      className='border-orange bg-orange text-sm-plus font-bold-plus text-ink-4 shadow-btn-orange inline-flex items-center gap-3.5 rounded-lg border px-4.5 py-3.5 transition-transform duration-200 hover:-translate-y-0.5'
                      href={link.href}>
                      {link.label} <span aria-hidden='true'>↗</span>
                    </a>
                  ))}
                </div>
              </article>
            ))}
            </div>
          </div>

          <div className='mb-12'>
            <h2 className='mb-6 text-xl font-semibold-plus tracking-tight-plus'>Editor Extensions</h2>
            <div className='max-md-nav:grid-cols-1 grid grid-cols-3 gap-4'>
              {extensionDownloads.map((item) => (
                <article
                  key={item.platform}
                  className='border-line-2 bg-feature-card rounded-xl border p-7'
                >
                  <div className='mb-6 text-3xl'><item.icon /></div>
                  <h2 className='tracking-tight-plus mb-2 text-xl'>{item.platform}</h2>
                  <p className='text-sm-plus text-muted-2 mb-6'>{item.description}</p>
                  <div className='flex flex-col gap-2'>
                    {item.links.map((link) => (
                      <a
                        key={link.label}
                        className='border-line-2 bg-bg-alt text-sm-plus font-bold-plus text-ink-2 inline-flex items-center gap-3.5 rounded-lg border px-4.5 py-3.5 transition-transform duration-200 hover:-translate-y-0.5'
                        href={link.href}
                      >
                        {link.label} <span aria-hidden='true'>↗</span>
                      </a>
                    ))}
                  </div>
                </article>
              ))}
            </div>
          </div>

          <div className='border-line-2 bg-panel-3 mt-12 rounded-xl border p-7'>
            <h3 className='mb-2 text-lg'>Install via Cargo</h3>
            <p className='text-sm-plus text-muted-2 mb-4'>Build and install the CLI from source</p>
            <code className='border-line-2 bg-bg-alt block rounded-lg border px-4 py-3 font-mono text-sm'>
              cargo install --git https://github.com/hieudoanm/mars mars-cli
            </code>
          </div>
        </section>
      </main>

      <Footer
        className={`${sectionShell} text-xs-plus text-faint-2 max-sm-nav:flex-col max-sm-nav:items-start flex items-center justify-between gap-5 py-7.5`}
      />
    </div>
  );
};

export default DownloadsPage;
