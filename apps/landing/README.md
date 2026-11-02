# Mars landing page

The landing page is the Next.js static site for the Mars project (`@mars/landing`).
It is deployed as static files suitable for GitHub Pages, Netlify, Cloudflare
Pages, or any static file host.

## Development

```bash
pnpm --filter @mars/landing dev
pnpm --filter @mars/landing check
```

## Build

```bash
pnpm --filter @mars/landing build
```

Set `NEXT_PUBLIC_SITE_URL` to the deployed origin when building so Open Graph
image URLs are absolute:

```bash
NEXT_PUBLIC_SITE_URL=https://example.com pnpm --filter @mars/landing build
```

The production output is written to `out/`. The site uses a static export with
trailing slashes and serves documentation at `/docs` from Markdown files in
`apps/landing/docs/`.

Brand assets are stored in `public/`. Keep those files synchronized with the
canonical files in `../../assets/svg` and `../../assets/images` when branding
changes.
