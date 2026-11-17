'use client';

import Link from 'next/link';
import { useMemo, useState, type FC } from 'react';
import type { Doc } from '@/lib/docs';
import { repositoryUrl } from '@/lib/site';
import { searchContent, toSearchableDocs } from '@/lib/search';

const sidebarLink =
  'rounded-lg px-2.5 py-[7px] text-sm-plus leading-[1.4] text-muted hover:bg-panel-2 hover:text-ink';

const chapterLabel = (chapter: string) => chapter.charAt(0).toUpperCase() + chapter.slice(1);

const DocsSearch: FC<Readonly<{ docs: Doc[] }>> = ({ docs }) => {
  const [query, setQuery] = useState('');

  const filteredDocs = useMemo(() => {
    const trimmed = query.trim();
    if (!trimmed) return docs;
    const results = searchContent(toSearchableDocs(docs), trimmed, docs.length);
    const bySlug = new Map(results.map((result) => [result.slug, result]));
    return docs.filter((doc) => bySlug.has(doc.slug));
  }, [docs, query]);

  const groupedDocs = useMemo(() => {
    if (query.trim()) return null;
    const groups = new Map<string, Doc[]>();
    for (const doc of filteredDocs) {
      const key = doc.chapter ?? '';
      const group = groups.get(key);
      if (group) group.push(doc);
      else groups.set(key, [doc]);
    }
    return [...groups.entries()];
  }, [filteredDocs, query]);

  return (
    <aside
      className='border-line bg-panel/80 max-lg-nav:static max-lg-nav:flex-row max-lg-nav:flex-wrap sticky top-7 flex flex-col gap-1 rounded-2xl border p-5'
      aria-label='Documentation navigation'>
      <p className='text-faint text-2xs tracking-caps-lg max-lg-nav:w-full m-0 mb-2.5 font-bold uppercase'>
        Documentation
      </p>
      <div className='mb-3 flex flex-col gap-1'>
        <label className='text-2xs text-muted font-bold uppercase' htmlFor='docs-search'>
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
      {groupedDocs
        ? groupedDocs.map(([chapter, chapterDocs]) => (
            <div className='flex flex-col gap-1' key={chapter || 'ungrouped'}>
              {chapter ? (
                <p className='text-faint text-2xs tracking-caps-lg mt-2 px-2.5 font-bold uppercase first:mt-0'>
                  {chapterLabel(chapter)}
                </p>
              ) : null}
              {chapterDocs.map((doc) => (
                <Link className={sidebarLink} href={`/docs/${doc.slug}`} key={doc.slug}>
                  {doc.title}
                </Link>
              ))}
            </div>
          ))
        : filteredDocs.map((doc) => (
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
