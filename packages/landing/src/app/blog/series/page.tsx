import Link from 'next/link';
import type { Metadata } from 'next';
import { getPosts } from '@/lib/blog';
import { siteName, siteUrl } from '@/lib/site';

export const metadata: Metadata = {
  title: `Series | ${siteName} Blog`,
  description:
    'Explore curated series on programming languages, notebook platforms, Jupyter kernels, data science libraries, and machine learning.',
  keywords: 'series, languages, notebooks, kernels, data science, machine learning, Project Mars',
  alternates: {
    canonical: `${siteUrl}/blog/series`,
  },
  openGraph: {
    title: `Series | ${siteName} Blog`,
    description:
      'Explore curated series on programming languages, notebook platforms, Jupyter kernels, data science libraries, and machine learning.',
    url: `${siteUrl}/blog/series`,
    siteName,
    images: [
      {
        url: `${siteUrl}/og-image.png`,
        width: 1200,
        height: 630,
        alt: 'Project Mars Blog Series',
      },
    ],
    type: 'website',
  },
  twitter: {
    card: 'summary_large_image',
    title: `Series | ${siteName} Blog`,
    description:
      'Explore curated series on programming languages, notebook platforms, Jupyter kernels, data science libraries, and machine learning.',
    images: [`${siteUrl}/og-image.png`],
  },
};

const series = [
  {
    slug: 'languages',
    title: 'Programming Languages',
    description:
      'Explore why Rust, Python, R, Julia, and C++ are essential for modern data science and how Project Mars supports them all.',
    posts: [
      'languages',
      'language-rust',
      'language-python',
      'language-r',
      'language-julia',
      'language-cpp',
    ],
  },
  {
    slug: 'notebooks',
    title: 'Notebook Platforms',
    description:
      'Compare Deepnote, Observable, Jupyter, and Project Mars to understand different approaches to interactive computing.',
    posts: [
      'notebooks',
      'notebook-mars',
      'notebook-jupyter',
      'notebook-deepnote',
      'notebook-observable',
      'notebook-hex',
    ],
  },
  {
    slug: 'kernels',
    title: 'Jupyter Kernels',
    description:
      "Explore language-specific Jupyter kernels and how they compare to Project Mars's native multi-language approach.",
    posts: [
      'kernels',
      'kernel-evcxr-jupyter',
      'kernel-ijulia',
      'kernel-xeus-cling',
      'kernel-irkernel',
      'kernel-matlab',
    ],
  },
  {
    slug: 'data-science',
    title: 'Data Science Libraries',
    description:
      'Compare Python and Rust data science libraries side-by-side, from NumPy to pandas to Polars.',
    posts: [
      'data-science-libraries',
      'data-science-library-numpy-vs-ndarray',
      'data-science-library-pandas-vs-polars',
      'data-science-library-scipy-vs-faer',
      'data-science-library-statsmodels-vs-statrs',
      'data-science-library-matplotlib-vs-plotters',
      'data-science-library-seaborn-vs-charming',
      'data-science-library-plotly-python-vs-rust',
      'data-science-library-jupyter-vs-mars',
      'data-science-library-networkx-vs-petgraph',
      'data-science-library-xarray-vs-ndarray',
    ],
  },
  {
    slug: 'machine-learning',
    title: 'Machine Learning Libraries',
    description:
      'Compare Python and Rust machine learning libraries from scikit-learn to PyTorch to XGBoost.',
    posts: [
      'machine-learning-libraries',
      'machine-learning-library-scikit-learn-vs-linfa',
      'machine-learning-library-pytorch-vs-tch-rs',
      'machine-learning-library-tensorflow-vs-tensorflow',
      'machine-learning-library-keras-vs-burn',
      'machine-learning-library-xgboost-vs-xgboost-rs',
      'machine-learning-library-lightgbm-vs-lightgbm',
      'machine-learning-library-catboost-vs-catboost-rs',
      'machine-learning-library-jax-vs-dfdx',
      'machine-learning-library-transformers-vs-rust-bert',
      'machine-learning-library-opencv-vs-opencv',
    ],
  },
  {
    slug: 'machine-learning-projects',
    title: 'Machine Learning Projects',
    description: 'Hands-on tutorials for 10 essential ML projects every beginner should know.',
    posts: [
      'machine-learning-projects',
      'machine-learning-project-iris-classification',
      'machine-learning-project-mnist-digit-recognition',
      'machine-learning-project-spam-classifier',
      'machine-learning-project-sentiment-analysis',
      'machine-learning-project-house-price-prediction',
      'machine-learning-project-customer-churn-prediction',
      'machine-learning-project-credit-card-fraud-detection',
      'machine-learning-project-movie-recommendation',
      'machine-learning-project-image-classification',
      'machine-learning-project-demand-forecasting',
    ],
  },
];

export default function SeriesPage() {
  const posts = getPosts();
  const postMap = new Map(posts.map((post) => [post.slug, post]));

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
        name: 'Blog',
        item: `${siteUrl}/blog`,
      },
      {
        '@type': 'ListItem',
        position: 3,
        name: 'Series',
        item: `${siteUrl}/blog/series`,
      },
    ],
  };

  return (
    <div>
      <script
        type='application/ld+json'
        dangerouslySetInnerHTML={{ __html: JSON.stringify(breadcrumbSchema) }}
      />
      <div className='mb-10'>
        <h1 className='text-ink text-3xl font-bold tracking-tight'>Series</h1>
        <p className='text-muted mt-2'>
          Curated collections of posts exploring languages, notebooks, kernels, and machine
          learning.
        </p>
      </div>

      <div className='space-y-8'>
        {series.map((seriesItem) => (
          <div key={seriesItem.slug} className='border-line bg-panel rounded-xl border p-6'>
            <h2 className='text-ink text-xl font-semibold'>{seriesItem.title}</h2>
            <p className='text-muted mt-2'>{seriesItem.description}</p>
            <div className='mt-4 space-y-2'>
              {seriesItem.posts.map((slug) => {
                const post = postMap.get(slug);
                if (!post) return null;
                return (
                  <Link
                    key={slug}
                    href={`/blog/${slug}`}
                    className='group border-line bg-bg hover:border-orange/60 hover:bg-panel-2 flex items-center justify-between rounded-lg border px-4 py-3 transition'>
                    <span className='text-ink group-hover:text-orange-2 font-medium'>
                      {post.title}
                    </span>
                    <span className='text-orange-2 text-sm'>→</span>
                  </Link>
                );
              })}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
