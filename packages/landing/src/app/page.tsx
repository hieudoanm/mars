import type { FC } from 'react';
import type { NextPage } from 'next';
import Navbar from '../components/layout/Navbar';
import Hero from '../components/home/Hero';
import Bands from '../components/home/Bands';
import WhyMars from '../components/home/WhyMars';
import Demo from '../components/home/Demo';
import Workflow from '../components/home/Workflow';
import BuiltDifferent from '../components/home/BuiltDifferent';
import Tools from '../components/home/Tools';
import GetStarted from '../components/home/GetStarted';
import Footer from '../components/layout/Footer';

const organizationSchema = {
  '@context': 'https://schema.org',
  '@type': 'Organization',
  name: 'Project Mars',
  url: 'https://hieudoanm.github.io/mars',
  logo: 'https://hieudoanm.github.io/mars/icons/mars-logo.png',
  description:
    'Project Mars is a Rust-powered interactive notebook platform for exploring ideas, executing code, and discovering possibilities.',
  sameAs: ['https://github.com/hieudoanm/mars'],
};

const websiteSchema = {
  '@context': 'https://schema.org',
  '@type': 'WebSite',
  name: 'Project Mars',
  url: 'https://hieudoanm.github.io/mars',
  description:
    'Project Mars is a Rust-powered interactive notebook platform for exploring ideas, executing code, and discovering possibilities.',
  publisher: {
    '@type': 'Organization',
    name: 'Project Mars',
    url: 'https://hieudoanm.github.io/mars',
  },
};

const softwareSchema = {
  '@context': 'https://schema.org',
  '@type': 'SoftwareApplication',
  name: 'Project Mars',
  applicationCategory: 'DeveloperApplication',
  operatingSystem: 'macOS, Windows, Linux',
  description:
    'A Rust-first interactive notebook platform with multi-language support, rich output rendering, and CLI automation.',
  url: 'https://hieudoanm.github.io/mars',
  downloadUrl: 'https://hieudoanm.github.io/mars/downloads/',
  offers: {
    '@type': 'Offer',
    price: '0',
    priceCurrency: 'USD',
  },
  featureList: [
    'Multi-language support: Rust, Python, R, Julia, C++',
    'Desktop application',
    'CLI for headless execution',
    'Rich output rendering',
    'Jupyter compatibility',
    'MCP integration',
    'Editor extensions for VS Code, Zed, JetBrains',
  ],
};

const speakableSchema = {
  '@context': 'https://schema.org',
  '@type': 'WebPage',
  name: 'Project Mars - Explore ideas. Execute code.',
  url: 'https://hieudoanm.github.io/mars',
  speakable: {
    '@type': 'SpeakableSpecification',
    cssSelector: ['h1', 'h2', '.prose'],
  },
};

const faqSchema = {
  '@context': 'https://schema.org',
  '@type': 'FAQPage',
  mainEntity: [
    {
      '@type': 'Question',
      name: 'What is Project Mars?',
      acceptedAnswer: {
        '@type': 'Answer',
        text: 'Project Mars is a Rust-powered interactive notebook platform for exploring ideas, executing code, and discovering possibilities. It supports multiple programming languages including Rust, Python, R, Julia, and C++.',
      },
    },
    {
      '@type': 'Question',
      name: 'Is Project Mars free and open source?',
      acceptedAnswer: {
        '@type': 'Answer',
        text: 'Yes, Project Mars is completely free and open source. You can download it from the releases page or build it from source on GitHub.',
      },
    },
    {
      '@type': 'Question',
      name: 'What programming languages does Project Mars support?',
      acceptedAnswer: {
        '@type': 'Answer',
        text: 'Project Mars supports Rust, Python, R, Julia, and C++ with native runtimes for each language. It also has Jupyter compatibility for importing and exporting .ipynb files.',
      },
    },
    {
      '@type': 'Question',
      name: 'How is Project Mars different from Jupyter?',
      acceptedAnswer: {
        '@type': 'Answer',
        text: 'Project Mars is an independent platform, not a Jupyter frontend or kernel. It has its own .mars document format, native multi-language support, desktop application, CLI automation, and MCP integration for AI assistants.',
      },
    },
    {
      '@type': 'Question',
      name: 'Can I use Project Mars for data science and machine learning?',
      acceptedAnswer: {
        '@type': 'Answer',
        text: 'Yes, Project Mars is designed for data science, machine learning, scientific computing, and education. It supports rich output rendering including charts, tables, LaTeX equations, and interactive visualizations.',
      },
    },
  ],
};

const Page: FC & NextPage = () => {
  return (
    <div className='relative overflow-hidden'>
      <script
        type='application/ld+json'
        dangerouslySetInnerHTML={{ __html: JSON.stringify(organizationSchema) }}
      />
      <script
        type='application/ld+json'
        dangerouslySetInnerHTML={{ __html: JSON.stringify(websiteSchema) }}
      />
      <script
        type='application/ld+json'
        dangerouslySetInnerHTML={{ __html: JSON.stringify(softwareSchema) }}
      />
      <script
        type='application/ld+json'
        dangerouslySetInnerHTML={{ __html: JSON.stringify(speakableSchema) }}
      />
      <script
        type='application/ld+json'
        dangerouslySetInnerHTML={{ __html: JSON.stringify(faqSchema) }}
      />
      <div className='animate-drift bg-hero-glow-1 pointer-events-none absolute top-20 right-1/12 h-130 w-130 rounded-full blur-xs' />
      <div className='animate-drift-reverse bg-hero-glow-2 pointer-events-none absolute top-150 -left-62 h-138 w-138 rounded-full blur-xs' />
      <Navbar />
      <main>
        <Hero />
        <Bands />
        <WhyMars />
        <Demo />
        <Workflow />
        <BuiltDifferent />
        <Tools />
        <GetStarted />
      </main>
      <Footer />
    </div>
  );
};

export default Page;
