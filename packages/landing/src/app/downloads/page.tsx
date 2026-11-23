import type { FC } from 'react';
import type { NextPage } from 'next';
import type { Metadata } from 'next';
import Footer from '@/components/layout/Footer';
import Navbar from '@/components/layout/Navbar';
import { repositoryUrl, siteUrl } from '@/lib/site';
import {
  cliDownloads,
  desktopDownloads,
  extensionDownloads,
  type DownloadItem,
} from '@/lib/downloads';
import {
  PiAppleLogo,
  PiWindowsLogo,
  PiLinuxLogo,
  PiCode,
  PiTerminal,
  PiToolbox,
} from 'react-icons/pi';

export const metadata: Metadata = {
  title: 'Downloads',
  description:
    'Download Project Mars desktop app for macOS, Windows, and Linux. Install VS Code, Zed, and JetBrains extensions.',
  alternates: {
    canonical: `${siteUrl}/downloads`,
  },
  openGraph: {
    title: 'Download Project Mars',
    description:
      'Get the Project Mars desktop application and editor extensions for your platform.',
    url: `${siteUrl}/downloads`,
    type: 'website',
  },
  twitter: {
    card: 'summary',
    title: 'Download Project Mars',
    description:
      'Get the Project Mars desktop application and editor extensions for your platform.',
  },
};

const sectionShell = 'mx-auto w-full max-w-shell px-6 max-sm-nav:px-4';

const desktopIcons: Record<string, FC> = {
  macOS: PiAppleLogo,
  Windows: PiWindowsLogo,
  Linux: PiLinuxLogo,
};

const extensionIcons: Record<string, FC> = {
  'VS Code': PiCode,
  Zed: PiTerminal,
  JetBrains: PiToolbox,
};

const cliIcons: Record<string, FC> = {
  'Linux x86_64': PiLinuxLogo,
  macOS: PiAppleLogo,
  'Windows x64': PiWindowsLogo,
};

function DownloadCard({ item, icon }: { item: DownloadItem; icon: FC }) {
  const Icon = icon;
  return (
    <article className='border-line-2 bg-feature-card rounded-xl border p-7'>
      <div className='mb-6 text-3xl'>
        <Icon />
      </div>
      <h3 className='tracking-tight-plus mb-2 text-xl'>{item.platform}</h3>
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
  );
}

function DownloadGrid({
  title,
  items,
  icons,
}: {
  title: string;
  items: DownloadItem[];
  icons: Record<string, FC>;
}) {
  return (
    <div className='mb-12'>
      <h2 className='font-semibold-plus tracking-tight-plus mb-6 text-xl'>{title}</h2>
      <div className='max-md-nav:grid-cols-1 grid grid-cols-3 gap-4'>
        {items.map((item) => (
          <DownloadCard key={item.platform} item={item} icon={icons[item.platform]!} />
        ))}
      </div>
    </div>
  );
}

const DownloadsPage: FC & NextPage = () => {
  const breadcrumbSchema = {
    '@context': 'https://schema.org',
    '@type': 'BreadcrumbList',
    itemListElement: [
      {
        '@type': 'ListItem',
        position: 1,
        name: 'Home',
        item: siteUrl,
      },
      {
        '@type': 'ListItem',
        position: 2,
        name: 'Downloads',
        item: `${siteUrl}/downloads`,
      },
    ],
  };

  return (
    <div className='bg-bg-alt text-ink-2 min-h-screen'>
      <script
        type='application/ld+json'
        dangerouslySetInnerHTML={{ __html: JSON.stringify(breadcrumbSchema) }}
      />
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
              Get Project Mars for your platform
            </h1>
            <p className='text-muted-2 text-lg'>
              Download the desktop app, CLI binaries, install extensions, or build from source.
            </p>
          </div>

          <DownloadGrid title='Desktop Application' items={desktopDownloads} icons={desktopIcons} />

          <div className='border-line-2 bg-panel-3 mb-12 rounded-xl border p-7'>
            <h2 className='mb-2 text-lg'>macOS: unsigned app workaround</h2>
            <p className='text-sm-plus text-muted-2'>
              The macOS builds are not signed or notarized, so Gatekeeper will block the first
              launch. Either right-click the app and choose{' '}
              <strong className='text-ink-2'>Open</strong>, or remove the quarantine attribute:
            </p>
            <code className='border-line-2 bg-bg-alt mt-4 block rounded-lg border px-4 py-3 font-mono text-sm'>
              xattr -dr com.apple.quarantine /Applications/Project\ Mars.app
            </code>
          </div>

          <DownloadGrid title='CLI' items={cliDownloads} icons={cliIcons} />

          <DownloadGrid
            title='Editor Extensions'
            items={extensionDownloads}
            icons={extensionIcons}
          />

          <div className='border-line-2 bg-panel-3 mt-12 rounded-xl border p-7'>
            <h3 className='mb-2 text-lg'>Install via script or Cargo</h3>
            <p className='text-sm-plus text-muted-2 mb-4'>
              Install prebuilt binaries, or build everything from source
            </p>
            <code className='border-line-2 bg-bg-alt mb-3 block rounded-lg border px-4 py-3 font-mono text-sm'>
              curl -fsSL
              https://raw.githubusercontent.com/hieudoanm/mars/master/scripts/sh/install.sh | bash
              -s -- --release
            </code>
            <code className='border-line-2 bg-bg-alt block rounded-lg border px-4 py-3 font-mono text-sm'>
              cargo install --git {repositoryUrl} mars-cli
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
