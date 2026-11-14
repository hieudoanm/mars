import type { Metadata } from "next";
import Link from "next/link";
import { notFound } from "next/navigation";
import Markdown from "@/components/shared/Markdown";
import { getDoc, getDocs } from "@/lib/docs";
import { siteName, siteUrl } from "@/lib/site";

export const dynamicParams = false;

export const generateStaticParams = () => {
  return getDocs().map((doc) => ({ slug: doc.slug }));
};

export const generateMetadata = async ({
  params,
}: {
  params: Promise<{ slug: string }>;
}): Promise<Metadata> => {
  const { slug } = await params;
  const doc = getDoc(slug);
  if (!doc) return {};

  const title = `${doc.title} | Mars Documentation`;
  const description = doc.description || `Learn about ${doc.title} in Mars documentation.`;
  const url = `${siteUrl}/docs/${doc.slug}`;

  return {
    title,
    description,
    alternates: {
      canonical: url,
    },
    openGraph: {
      title,
      description,
      url,
      siteName,
      type: 'article',
    },
    twitter: {
      card: 'summary',
      title,
      description,
    },
  };
};

type DocPageProps = Readonly<{
  params: Promise<{ slug: string }>;
}>;

const DocPage = async ({ params }: DocPageProps) => {
  const { slug } = await params;
  const doc = getDoc(slug);
  if (!doc) notFound();

  const docs = getDocs();
  const currentIndex = docs.findIndex((d) => d.slug === slug);
  const previous = currentIndex > 0 ? docs[currentIndex - 1] : undefined;
  const next = currentIndex >= 0 && currentIndex < docs.length - 1 ? docs[currentIndex + 1] : undefined;

  const techArticleSchema = {
    '@context': 'https://schema.org',
    '@type': 'TechArticle',
    headline: doc.title,
    description: doc.description || `Learn about ${doc.title} in Mars documentation.`,
    author: {
      '@type': 'Organization',
      name: siteName,
      url: siteUrl,
    },
    publisher: {
      '@type': 'Organization',
      name: siteName,
      url: siteUrl,
    },
    mainEntityOfPage: {
      '@type': 'WebPage',
      '@id': `${siteUrl}/docs/${doc.slug}`,
    },
    isPartOf: {
      '@type': 'WebSite',
      name: 'Mars Documentation',
      url: `${siteUrl}/docs`,
    },
  };

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
      {
        '@type': 'ListItem',
        position: 3,
        name: doc.title,
        item: `${siteUrl}/docs/${doc.slug}`,
      },
    ],
  };

  return (
    <div>
      <script
        type='application/ld+json'
        dangerouslySetInnerHTML={{ __html: JSON.stringify(techArticleSchema) }}
      />
      <script
        type='application/ld+json'
        dangerouslySetInnerHTML={{ __html: JSON.stringify(breadcrumbSchema) }}
      />
      <Markdown content={doc.content} />
      <nav className="mt-12 flex items-center justify-between gap-4 border-t border-line pt-6">
        <div className="flex-1">
          {previous ? (
            <Link
              className="group flex items-center gap-2 rounded-lg border border-line px-4 py-2 text-sm text-ink transition hover:bg-panel-2"
              href={`/docs/${previous.slug}`}
            >
              <span aria-hidden="true">←</span>
              <span className="flex flex-col">
                <span className="text-xs text-muted">Previous</span>
                <span className="font-medium">{previous.title}</span>
              </span>
            </Link>
          ) : null}
        </div>
        <div className="flex-1 text-right">
          {next ? (
            <Link
              className="group ml-auto flex items-center justify-end gap-2 rounded-lg border border-line px-4 py-2 text-sm text-ink transition hover:bg-panel-2"
              href={`/docs/${next.slug}`}
            >
              <span className="flex flex-col items-end">
                <span className="text-xs text-muted">Next</span>
                <span className="font-medium">{next.title}</span>
              </span>
              <span aria-hidden="true">→</span>
            </Link>
          ) : null}
        </div>
      </nav>
    </div>
  );
};

export default DocPage;
