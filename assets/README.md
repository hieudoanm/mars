# Mars Assets

This directory contains visual assets for the Mars project, including logos, icons, and screenshots used across the repository, documentation, and applications.

## Structure

```
assets/
├── images/           # PNG raster images
│   ├── mars-logo.png
│   ├── mars-logo-file.png
│   ├── mars-logo-folder-close.png
│   ├── mars-logo-folder-open.png
│   └── mars-og.png
├── svg/              # SVG vector graphics
│   ├── mars-logo.svg
│   ├── mars-logo-file.svg
│   ├── mars-logo-folder-close.svg
│   ├── mars-logo-folder-open.svg
│   └── mars-og.svg
├── screenshots/      # Application screenshots
└── svg-to-png.sh     # Script to convert SVG to PNG
```

## Assets

### Logo Variants

#### mars-logo.svg / mars-logo.png

The primary Mars logo used in:

- Repository README
- Landing site header
- Desktop application window
- Documentation pages

#### mars-logo-file.svg / mars-logo-file.png

File icon for `.mars` files used in:

- VS Code extension file icon theme
- Desktop application file explorer
- IDE extensions

#### mars-logo-folder-close.svg / mars-logo-folder-close.png

Folder icon (closed) for Mars projects used in:

- Desktop application file explorer
- Project directory visualization

#### mars-logo-folder-open.svg / mars-logo-folder-open.png

Folder icon (open) for Mars projects used in:

- Desktop application file explorer
- Project directory visualization

#### mars-og.svg / mars-og.png

Open Graph image for social media sharing used in:

- Landing site meta tags
- Social media previews
- GitHub repository social preview

## Asset Formats

### SVG (Scalable Vector Graphics)

SVG files are the source of truth for all Mars graphics. They are:

- Scalable without quality loss
- Editable with vector graphics tools
- Smaller file size for simple graphics
- Used in web applications and documentation

**Location:** `svg/`

### PNG (Portable Network Graphics)

PNG files are rasterized versions for:

- Icons that need fixed pixel dimensions
- Fallback for environments without SVG support
- Open Graph images for social media
- Desktop application resources

**Location:** `images/`

## Generating PNG Assets

The `svg-to-png.sh` script converts SVG files to PNG format at multiple resolutions.

### Usage

```bash
cd assets
./svg-to-png.sh
```

### Requirements

- ImageMagick (`convert` command)
- Inkscape (optional, for better rendering)

### Install Dependencies

**macOS:**

```bash
brew install imagemagick
brew install inkscape
```

**Linux (Ubuntu/Debian):**

```bash
sudo apt install imagemagick inkscape
```

**Linux (Fedora):**

```bash
sudo dnf install imagemagick inkscape
```

## Screenshots

The `screenshots/` directory is reserved for application screenshots used in:

- Landing site
- Documentation
- Release notes
- Marketing materials

### Adding Screenshots

1. Take a screenshot of the Mars desktop application
2. Place it in `screenshots/`
3. Use a descriptive filename: `desktop-overview.png`, `cell-execution.png`, etc.
4. Update relevant documentation to reference the new screenshot

### Screenshot Guidelines

- Use the dark theme (default)
- Show meaningful content (not empty notebooks)
- Maintain consistent aspect ratio (16:9 or 4:3)
- Include full window for context
- Ensure text is readable at reduced sizes

## Usage Across Repository

### Repository README

- Primary logo: `images/mars-logo.png`
- Used in: `/README.md`

### Landing Site

- Primary logo: `svg/mars-logo.svg`
- Open Graph: `images/mars-og.png`
- Used in: `packages/landing/`

### Desktop Application

- Window icon: `images/mars-logo.png`
- File icon: `images/mars-logo-file.png`
- Folder icons: `images/mars-logo-folder-*.png`
- Used in: `packages/desktop/`

### VS Code Extension

- File icon: `images/mars-logo-file.png`
- Used in: `extensions/vscode/`

### Zed Extension

- File icon: `images/mars-logo-file.png`
- Used in: `extensions/zed/`

### JetBrains Extension

- File icon: `images/mars-logo-file.png`
- Used in: `extensions/jetbrains/`

## Branding Guidelines

### Colors

The Mars brand uses the following primary colors:

- **Mars Red** (primary accent): `#E74C3C` or similar
- **Dark Gray** (backgrounds): `#1E1E1E` or `#2D2D2D`
- **Light Gray** (text): `#D4D4D4` or `#E0E0E0`

### Typography

- Use the logo with sufficient white space
- Maintain aspect ratio (do not stretch)
- Minimum size for readability: 32px height
- For print: use SVG source

### Usage Rules

- **Do not modify** the logo's proportions
- **Do not recolor** the logo elements
- **Do not add** drop shadows or effects
- **Do use** SVG format for web when possible
- **Do use** appropriate opacity on dark backgrounds

## Modifying Assets

### Editing SVG Files

1. Use a vector graphics editor (Inkscape, Figma, Adobe Illustrator)
2. Edit files in the `svg/` directory
3. Maintain the original design intent
4. Regenerate PNG files using `svg-to-png.sh`

### Creating New Assets

1. Create new SVG files in `svg/`
2. Follow existing naming conventions
3. Ensure scalability (use vector paths, not raster images)
4. Generate PNG versions if needed
5. Update this README with the new asset

## Version Control

### Committing Assets

- Commit both SVG and PNG files
- Use descriptive commit messages
- If regenerating PNGs, mention the reason
- Consider using Git LFS for large assets

### Large Assets

If assets become very large (>100KB), consider:

- Using Git LFS: `git lfs track "*.png"`
- Optimizing SVG files (remove unnecessary metadata)
- Reducing PNG resolution where appropriate

## Asset Optimization

### SVG Optimization

```bash
# Install svgo
npm install -g svgo

# Optimize SVG files
svgo svg/mars-logo.svg
```

### PNG Optimization

```bash
# Install optipng
brew install optipng  # macOS
sudo apt install optipng  # Linux

# Optimize PNG files
optipng images/mars-logo.png
```

## Related Documentation

- [Branding documentation](../docs/02-branding.md)
- [Release documentation](../docs/14-release.md)
- [Landing site](../packages/landing/)
- [Desktop application](../packages/desktop/)

## Contributing

When contributing visual assets:

1. Provide both SVG and PNG versions
2. Follow the naming conventions
3. Ensure assets work on both light and dark backgrounds
4. Test in the actual application context
5. Update this README with new assets
6. Regenerate PNGs if SVG source changes

## License

All Mars assets are part of the Mars project and follow the same license as the repository. See the repository LICENSE file for details.

## Support

For questions about assets or branding:

- Open an issue on the Mars repository
- Review the [branding documentation](../docs/02-branding.md)
- Check the [landing site documentation](../packages/landing/md/docs/)
