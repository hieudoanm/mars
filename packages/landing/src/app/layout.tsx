import type { Metadata } from 'next';
import type { FC, ReactNode } from 'react';
import marsOg from '@/assets/png/mars-og.png';
import './globals.css';

export const metadata: Metadata = {
  metadataBase: new URL(process.env.NEXT_PUBLIC_SITE_URL ?? 'https://hieudoanm.github.io/mars'),
  title: {
    default: 'Mars - Explore ideas. Execute code.',
    template: '%s | Mars',
  },
  description:
    'Mars is a Rust-powered interactive notebook for exploring ideas, executing code, and discovering possibilities.',
  applicationName: 'Mars',
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
    title: 'Mars - Explore ideas. Execute code.',
    description:
      'A Rust-first interactive notebook with isolated execution, rich output, and its own native document format.',
    images: [{ url: marsOg.src, width: 1200, height: 630, alt: 'Mars' }],
    type: 'website',
    siteName: 'Mars',
  },
  twitter: {
    card: 'summary_large_image',
    title: 'Mars - Explore ideas. Execute code.',
    description:
      'A Rust-first interactive notebook with isolated execution, rich output, and its own native document format.',
    images: [marsOg.src],
  },
};

type RootLayoutProps = Readonly<{ children: ReactNode }>;

const RootLayout: FC<RootLayoutProps> = ({ children }) => {
  return (
    <html lang='en'>
      <body>{children}</body>
    </html>
  );
};

export default RootLayout;
