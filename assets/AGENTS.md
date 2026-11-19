# assets/ — Visual Assets

Rules for editing files under `assets/`. For the full catalog of what each
file is and where it is used, see [`README.md`](README.md) — do not duplicate
that inventory here.

## Source of truth

- **SVG is canonical. PNG is generated.** Never hand-edit a `.png`; edit the
  sibling `.svg` and regenerate.
- After editing any `*/svg/*.svg`, run `assets/svg-to-png.sh` (requires
  `rsvg-convert`) so the `*/png/` exports stay in sync. CI does not check
  this — a stale PNG is a silent inconsistency.
- Keep SVGs self-contained: no external fonts, images, scripts, or network
  references. `logo-mono.svg` relies on `currentColor` — preserve that.

## Brand artwork

- Do not modify logo proportions, recolor elements, add effects, or redraw
  brand marks. `branding/` changes propagate to the whole project.
- When brand artwork changes, update `assets/branding/` first, then sync the
  package-local copies consumed by `packages/landing`, `packages/desktop`,
  `extensions/vscode`, and the repo README (see "Package-local copies" in
  `README.md` for the exact destinations).
- `sync-assets.sh` refreshes `packages/landing/public/` (logo.svg, og.png at
  1200x630, favicon.ico via ImageMagick). Run it after branding edits that
  affect the landing site.

## Demo assets

- `demo/svg-to-gif.sh` regenerates `demo/{svg,png,gif}` from
  `scripts/demo-sessions/mars.json` via the shared generator in the sibling
  `hieudoanm` checkout. It requires `rsvg-convert` and `ffmpeg`, and fails
  outside that layout — do not "fix" it to run standalone.
- `demo/png/desktop-*.png` screenshots are captured by
  `scripts/sh/screenshot.sh`; they are not produced by `svg-to-png.sh`.

## Conventions

- Directory layout is `assets/<group>/{svg,png}/` — keep new artwork in that
  shape so `svg-to-png.sh` picks it up automatically.
- Language icons live in `icons/languages/`; keep SVG viewBoxes tight around
  the glyph so 28px downscaling stays crisp.
- Filenames are kebab-case and match across `svg/` and `png/`.
- When adding or renaming an asset, update `README.md` in the same change —
  the README is the index of record.

## Validation

```bash
assets/svg-to-png.sh   # regenerates all PNGs; exits non-zero on failure
git status             # PNGs should show as modified, never as hand-edited
```

There is no automated asset lint; visual review of the rendered PNG is the
check. Assets themselves have no test suite — the repository-wide 80%
line/function coverage rule applies to code, not artwork.
