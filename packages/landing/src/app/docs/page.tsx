import type { FC } from 'react';
import type { NextPage } from 'next';
import { notFound } from 'next/navigation';
import Markdown from '@/components/shared/Markdown';
import { getDoc } from '@/lib/docs';
import { siteUrl } from '@/lib/site';

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
      name: 'Documentation',
      item: `${siteUrl}/docs`,
    },
  ],
};

const DocsPage: FC & NextPage = () => {
  const doc = getDoc('index');
  if (!doc) notFound();
  return (
    <div>
      <script
        type='application/ld+json'
        dangerouslySetInnerHTML={{ __html: JSON.stringify(breadcrumbSchema) }}
      />
      <Markdown content={doc.content} />
    </div>
  );
};

export default DocsPage;
