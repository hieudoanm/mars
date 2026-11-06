import type { FC } from 'react';
import type { NextPage } from 'next';
import { notFound } from 'next/navigation';
import Markdown from '@/components/shared/Markdown';
import { getDoc } from '@/lib/docs';

const DocsPage: FC & NextPage = () => {
  const doc = getDoc('index');
  if (!doc) notFound();
  return <Markdown content={doc.content} />;
};

export default DocsPage;
