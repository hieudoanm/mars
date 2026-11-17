import type { Metadata } from 'next';
import type { FC, ReactNode } from 'react';
import Footer from '@/components/layout/Footer';
import Navbar from '@/components/layout/Navbar';

export const metadata: Metadata = {
  title: 'Blog',
  description: 'Essays on Rust, notebooks, data science, and the case for Project Mars.',
};

const sectionShell = 'mx-auto w-full max-w-shell px-6 max-sm-nav:px-4';

type BlogLayoutProps = Readonly<{ children: ReactNode }>;

const BlogLayout: FC<BlogLayoutProps> = ({ children }) => {
  return (
    <div className='flex min-h-screen flex-col'>
      <Navbar
        className='max-w-header max-sm-nav:px-4 max-sm-nav:py-5 relative z-20 mx-auto flex w-full items-center justify-between px-6 py-7'
        showHome
      />

      <main className={`${sectionShell} max-lg-nav:py-8 max-lg-nav:pb-18 flex-1 py-14 pb-22`}>
        <div className='border-line bg-panel/80 rounded-2.5xl max-lg-nav:px-5.5 max-lg-nav:py-7 min-w-0 border px-12 py-10'>
          {children}
        </div>
      </main>

      <Footer
        className={`${sectionShell} text-xs-plus text-faint-2 max-sm-nav:flex-col max-sm-nav:items-start flex items-center justify-between gap-5 py-7.5 pb-11`}
      />
    </div>
  );
};

export default BlogLayout;
