import type { Metadata } from 'next';
import type { FC, ReactNode } from 'react';
import DocsSearch from '../../components/docs/DocsSearch';
import Footer from '@/components/layout/Footer';
import Navbar from '@/components/layout/Navbar';
import { getDocs } from '@/lib/docs';

export const metadata: Metadata = {
  title: 'Documentation',
  description:
    'Guides for installing Project Mars, running notebooks, understanding the .mars format, and using the CLI.',
  openGraph: {
    title: 'Project Mars Documentation',
    description:
      'Guides for installing Project Mars, running notebooks, understanding the .mars format, and using the CLI.',
    type: 'website',
  },
  twitter: {
    card: 'summary',
    title: 'Project Mars Documentation',
    description:
      'Guides for installing Project Mars, running notebooks, understanding the .mars format, and using the CLI.',
  },
};

const sectionShell = 'mx-auto w-full max-w-shell px-6 max-sm-nav:px-4';

type DocsLayoutProps = Readonly<{ children: ReactNode }>;

const DocsLayout: FC<DocsLayoutProps> = ({ children }) => {
  const docs = getDocs();

  return (
    <div className='flex min-h-screen flex-col'>
      <Navbar
        className='max-w-header max-sm-nav:px-4 max-sm-nav:py-5 relative z-20 mx-auto flex w-full items-center justify-between px-6 py-7'
        showHome
      />

      <main
        className={`${sectionShell} max-lg-nav:gap-5.5 max-lg-nav:py-8 max-lg-nav:pb-18 lg-nav:grid-cols-12 grid flex-1 grid-cols-1 items-start gap-12 py-14 pb-22`}>
        <div className='lg-nav:col-span-3'>
          <DocsSearch docs={docs} />
        </div>
        <div className='border-line bg-panel/80 rounded-2.5xl max-lg-nav:px-5.5 max-lg-nav:py-7 lg-nav:col-span-9 min-w-0 border px-12 py-10'>
          {children}
        </div>
      </main>

      <Footer
        className={`${sectionShell} text-xs-plus text-faint-2 max-sm-nav:flex-col max-sm-nav:items-start flex items-center justify-between gap-5 py-7.5 pb-11`}
      />
    </div>
  );
};

export default DocsLayout;
