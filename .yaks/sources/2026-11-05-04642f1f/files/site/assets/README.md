# Landing page assets

Every image the page uses lives here, at a fixed path, so new graphics replace files without touching `index.html`. The emblem and icons come from the YASS logo (transparent background); the social card is the YASS brand graphic; `favicon.svg` is a simplified vector of the hex. A placeholder carries the marker `yass-placeholder` (an SVG comment, a PNG `tEXt` chunk, or WebP XMP metadata), and `tests/site.sh --deploy`, which the Pages workflow runs before it publishes, fails while any file still has it. Real graphics shouldn't contain the marker.

| File | Required | Spec | Where it shows |
|---|---|---|---|
| `emblem.webp` | yes | square, at least 460×460, ideally 600×600, transparent background; the hex emblem trimmed close to its edges (the page draws it at up to 240 px) | top of the hero, above the YASS! wordmark (which is HTML text, not an image) |
| `favicon.svg` | yes | square; reads at 16 px (the hex emblem alone, without the robot's detail, reads best) | browser tab |
| `favicon-32.png` | yes | 32×32 | browser tab, for browsers without SVG favicons |
| `apple-touch-icon.png` | yes | 180×180, opaque (no transparency) | home-screen icon on iOS |
| `og.png` | yes | 1200×630; keep text inside the central 1000×500, since some apps crop | the preview card when someone shares a link to the page |

`tests/site.sh` checks that the required files exist, that the page references them, and that the images have the sizes above. Everything the page loads, except `og.png`, has to stay under 150 KB in total, so prefer SVG and compress PNGs.
