import type { Metadata } from 'next';
import type { FC, ReactNode } from 'react';
import marsOg from '@/assets/png/mars-og.png';
import './globals.css';

export const metadata: Metadata = {
  metadataBase: new URL(process.env.NEXT_PUBLIC_SITE_URL ?? 'https://hieudoanm.github.io/mars'),
  title: {
    default: 'Project Mars - Explore ideas. Execute code.',
    template: '%s | Project Mars',
  },
  description:
    'Project Mars is a Rust-powered interactive notebook for exploring ideas, executing code, and discovering possibilities.',
  keywords: [
    'interactive notebook',
    'Jupyter alternative',
    'Rust',
    'reproducible computing',
    'data science',
    'machine learning',
    'notebook format',
    'MCP server',
  ],
  authors: [{ name: 'hieudoan', url: 'https://github.com/hieudoanm/mars' }],
  category: 'software',
  applicationName: 'Project Mars',
  manifest: '/manifest.json',
  alternates: {
    languages: {
      'en': 'https://hieudoanm.github.io/mars',
      'x-default': 'https://hieudoanm.github.io/mars',
    },
  },
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      'max-video-preview': -1,
      'max-image-preview': 'large',
      'max-snippet': -1,
    },
  },
  icons: {
    icon: [
      { url: '/icons/mars-logo.svg', type: 'image/svg+xml' },
      { url: '/icons/mars-logo.png', type: 'image/png', sizes: '512x512' },
    ],
    apple: [{ url: '/icons/mars-logo.png', sizes: '512x512' }],
  },
  openGraph: {
    title: 'Project Mars - Explore ideas. Execute code.',
    description:
      'A Rust-first interactive notebook with isolated execution, rich output, and its own native document format.',
    images: [{ url: marsOg.src, width: 1200, height: 630, alt: 'Project Mars' }],
    type: 'website',
    siteName: 'Project Mars',
  },
  twitter: {
    card: 'summary_large_image',
    title: 'Project Mars - Explore ideas. Execute code.',
    description:
      'A Rust-first interactive notebook with isolated execution, rich output, and its own native document format.',
    images: [marsOg.src],
  },
};

const JSON_LD = {
  '@context': 'https://schema.org',
  '@graph': [
    {
      '@type': 'WebSite',
      name: 'Project Mars',
      url: 'https://hieudoanm.github.io/mars',
      description:
        'Project Mars is a Rust-powered interactive notebook for exploring ideas, executing code, and discovering possibilities.',
    },
    {
      '@type': 'SoftwareApplication',
      name: 'Project Mars',
      applicationCategory: 'DeveloperApplication',
      description:
        'Rust-powered interactive notebook with isolated execution, rich output, and a native .mars document format.',
      url: 'https://hieudoanm.github.io/mars',
      codeRepository: 'https://github.com/hieudoanm/mars',
      offers: { '@type': 'Offer', price: '0', priceCurrency: 'USD' },
    },
  ],
};

type RootLayoutProps = Readonly<{ children: ReactNode }>;

const RootLayout: FC<RootLayoutProps> = ({ children }) => {
  return (
    <html lang='en'>
      <body>
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: JSON.stringify(JSON_LD) }}
        />
        {children}
      </body>
    </html>
  );
};

export default RootLayout;
