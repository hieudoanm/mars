# Project Mars branding assets

The project branding is built around a deep charcoal foundation with Project Mars red,
orange, and warm cream accents. The planet represents exploration. The primary
logo uses a white `M` as a notebook/code mark; the file logo is the simplified
ringless planet mark without the `M`.

Assets:

- [`assets/branding/svg/logo.svg`](../assets/branding/svg/logo.svg) — square 512×512 primary
  project logo.
- [`assets/branding/svg/logo-file.svg`](../assets/branding/svg/logo-file.svg) — square
  512×512 `.mars` notebook file icon.
- [`assets/branding/svg/logo-folder-close.svg`](../assets/branding/svg/logo-folder-close.svg)
  and [`assets/branding/svg/logo-folder-open.svg`](../assets/branding/svg/logo-folder-open.svg)
  — square 512×512 orange folder icons (closed/open states) for notebook
  collections, with the planet mark at the bottom right.
- [`assets/branding/svg/og.svg`](../assets/branding/svg/og.svg) — 1280×640 Open Graph/social
  preview image.

PNG exports are available under [`assets/branding/png/`](../assets/branding/png/) at the
same dimensions. The desktop app uses copies under
`packages/desktop/public/branding/`, and the VS Code package includes copies under
`extensions/vscode/assets/`.

All SVGs are self-contained with no external fonts, images, or network
dependencies. Keep the square marks on dark or transparent surfaces; use the
Open Graph asset as a full-bleed social preview.
