# Mars landing site

The Mars landing site is a static Next.js application for project discovery,
downloads, documentation, and blog posts. It is designed to deploy to GitHub
Pages or any static host—no Node.js server is required at runtime.

## Features

- Responsive Mars product pages, downloads, blog, and documentation routes.
- Markdown-backed documentation and posts with generated navigation and search.
- Static export with trailing-slash routes and unoptimized images for portable
  hosting.
- Accessible dark-theme navigation, mobile drawer, focus handling, and reduced
  motion support.
- Deployment-ready metadata, Open Graph assets, favicon, manifest, sitemap,
  robots, humans, and service-worker files.

## Requirements

Use the workspace's Corepack-managed pnpm version and a supported Node.js
release. Install dependencies from the repository root:

```bash
corepack enable
pnpm install
```

## Development

Start the local development server:

```bash
pnpm --filter @mars/landing dev
```

The site is available at `http://localhost:3000` by default. For package-local
development, run the same scripts from this directory with `pnpm`.

## Content and routes

| Location          | Purpose                                                             |
| ----------------- | ------------------------------------------------------------------- |
| `src/app/`        | Next.js App Router pages, layouts, metadata, loading, and error UI  |
| `src/components/` | Reusable home, docs, blog, layout, and shared UI components         |
| `src/lib/`        | Markdown loaders, search helpers, and site constants                |
| `md/docs/`        | Ordered documentation source files                                  |
| `md/posts/`       | Blog post source files                                              |
| `public/`         | Public logos, social image, PWA metadata, crawler files, and worker |

Documentation and blog source is Markdown. Keep the existing front matter and
filenames stable when possible, because they define route slugs, sort order,
navigation, and metadata. Build content transformations in `src/lib/`, not in
React components.

## Static deployment

`next.config.ts` enables `output: 'export'`, trailing slashes, and a production
base path of `/mars`. The production build writes static files to `out/`.

Set `NEXT_PUBLIC_SITE_URL` to the deployed origin when building metadata that
uses absolute URLs:

```bash
NEXT_PUBLIC_SITE_URL=https://example.com pnpm --filter @mars/landing build
```

Preview the generated site locally:

```bash
pnpm --filter @mars/landing serve
```

If the deployment base path changes, update `next.config.ts`, route metadata,
the sitemap, and public-asset links together.

## Validation

```bash
pnpm --filter @mars/landing check
pnpm --filter @mars/landing lint
pnpm --filter @mars/landing test
pnpm --filter @mars/landing build
```

Do not commit generated `.next/`, `out/`, coverage output, or TypeScript build
information. See [AGENTS.md](AGENTS.md) for component, Markdown, asset, and
accessibility conventions.
