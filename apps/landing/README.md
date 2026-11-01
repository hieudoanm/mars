# Mars landing page

This is the React/Vite marketing page for Mars. It is the
`@mars/landing` workspace package and produces static files suitable for
GitHub Pages, Netlify, Cloudflare Pages, or any static file host.

```bash
pnpm --filter @mars/landing build
```

For local development:

```bash
pnpm --filter @mars/landing check
pnpm --filter @mars/landing dev
```

The production output is written to `dist/`. The page uses local copies of the
shared project assets under `assets/`; keep those files synchronized with the
canonical files in `../../assets/svg` and `../../assets/images` when branding
changes.

The page uses copies of the shared project assets under `assets/`. Keep those
files synchronized with the canonical files in `../../assets/svg` and
`../../assets/images` when branding changes.
