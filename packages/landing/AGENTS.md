# Mars landing site

Scope: `packages/landing/**`. Follow the repository-level `AGENTS.md` first; these
rules apply to the public documentation and marketing site.

## Architecture and content

- This is a Next.js 16 App Router site exported as static files. Keep every
  route compatible with static export: do not add server-only runtime state,
  API routes, middleware requirements, or request-dependent rendering without
  first changing the deployment model deliberately.
- Route modules belong in `src/app/`; reusable page sections live in
  `src/components/`; parsing, searching, and site constants belong in `src/lib/`.
  Keep Markdown and search transformations out of React components.
- Documentation source is in `md/docs/` and posts in `md/posts/`. Preserve
  filename ordering and front matter conventions so `docs.ts` and `blog.ts` can
  generate stable routes, navigation, and metadata.
- Use `src/lib/site.ts` for the site name, repository URL, and deployment URL.
  Keep `NEXT_PUBLIC_SITE_URL` support intact and avoid hard-coded production
  origins in components or metadata.
- Prefer Server Components. Add `'use client'` only when browser state, event
  handlers, or client-only libraries genuinely require it; isolate client
  interactivity in the smallest component possible.
- Keep production source files at or below 200 lines where practical (220 is
  the absolute sensible exception). Split sections, data transforms, or shared
  controls instead of creating page-sized components.

## Public assets, accessibility, and security

- Files in `public/` are public, deployment-ready assets. Keep logo, favicon,
  Open Graph, manifest, sitemap, robots, humans, and service-worker references
  consistent when changing branding or routes. Do not reference assets from
  `src/assets/` in route metadata.
- Use `next/image` and `next/link` where appropriate, maintain meaningful image
  alt text, keyboard-operable navigation, semantic headings, and visible focus
  states. Preserve the mobile drawer's accessible labels and Escape handling.
- Markdown is content, not trusted HTML. Continue rendering it through the
  existing `Markdown` component and its constrained remark setup; do not enable
  arbitrary raw HTML or client-side script execution.
- Maintain the dark Mars design system in `globals.css`; avoid style duplication
  and unnecessary animations. Respect reduced-motion behaviour for new motion.
- Treat route params and front matter as untrusted input: validate/narrow
  external values and use `unknown`, never `any`.

## Validation

Run scoped checks from the repository root:

```bash
pnpm --filter @mars/landing check
pnpm --filter @mars/landing lint
pnpm --filter @mars/landing test
pnpm --filter @mars/landing build
```

Set `NEXT_PUBLIC_SITE_URL` when validating production metadata or Open Graph
URLs. Inspect the static `out/` result after changes to routing, public assets,
or metadata. Do not commit `.next/`, `out/`, coverage output, or TypeScript
build-info files.
