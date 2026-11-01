'use client';

import Link from 'next/link';
import { useMemo, useState, type FC } from 'react';
import type { Doc } from '@/lib/docs';
import { repositoryUrl } from '@/lib/site';
import { searchContent, toSearchableDocs } from '@/lib/search';

const sidebarLink =
  'rounded-lg px-2.5 py-[7px] text-sm-plus leading-[1.4] text-muted hover:bg-panel-2 hover:text-ink';

const DocsSearch: FC<Readonly<{ docs: Doc[] }>> = ({ docs }) => {
  const [query, setQuery] = useState('');

  const filteredDocs = useMemo(() => {
    const trimmed = query.trim();
    if (!trimmed) return docs;
    const results = searchContent(toSearchableDocs(docs), trimmed, docs.length);
    const bySlug = new Map(results.map((result) => [result.slug, result]));
    return docs.filter((doc) => bySlug.has(doc.slug));
  }, [docs, query]);

  return (
    <aside
      className='border-line bg-panel/80 sticky top-7 flex flex-col gap-1 rounded-2xl border p-5 max-lg-nav:static max-lg-nav:flex-row max-lg-nav:flex-wrap'
      aria-label='Documentation navigation'>
      <p className='text-faint m-0 mb-2.5 text-2xs font-bold tracking-caps-lg uppercase max-lg-nav:w-full'>
        Documentation
      </p>
      <div className='mb-3 flex flex-col gap-1'>
        <label className='text-2xs font-bold text-muted uppercase' htmlFor='docs-search'>
          Search
        </label>
        <input
          id='docs-search'
          aria-label='Search docs'
          className='border-line bg-bg focus:border-orange rounded-lg border px-3 py-2 text-sm focus:outline-none'
          placeholder='Search by title or description'
          value={query}
          onChange={(event) => setQuery(event.currentTarget.value)}
        />
      </div>
      <Link className={sidebarLink} href='/docs'>
        Overview
      </Link>
      {filteredDocs.map((doc) => (
        <Link className={sidebarLink} href={`/docs/${doc.slug}`} key={doc.slug}>
          {doc.title}
        </Link>
      ))}
      <a className={sidebarLink} href={`${repositoryUrl}/tree/main/docs`}>
        Repository docs <span aria-hidden='true'>↗</span>
      </a>
    </aside>
  );
};

export default DocsSearch;
