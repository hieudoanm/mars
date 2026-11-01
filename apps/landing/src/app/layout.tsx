import type { Metadata } from 'next';
import type { ReactNode } from 'react';
import './globals.css';

export const metadata: Metadata = {
  metadataBase: new URL(process.env.NEXT_PUBLIC_SITE_URL ?? 'http://localhost:4173'),
  title: {
    default: 'Mars - Explore ideas. Execute code.',
    template: '%s | Mars',
  },
  description:
    'Mars is a Rust-powered interactive notebook for exploring ideas, executing code, and discovering possibilities.',
  manifest: '/manifest.json',
  icons: {
    icon: '/images/svg/mars-file-logo.svg',
  },
  openGraph: {
    title: 'Mars - Explore ideas. Execute code.',
    description:
      'A Rust-first interactive notebook with isolated execution, rich output, and its own native document format.',
    images: ['/images/png/mars-og.png'],
  },
};

export default function RootLayout({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <html lang='en'>
      <body>{children}</body>
    </html>
  );
}
