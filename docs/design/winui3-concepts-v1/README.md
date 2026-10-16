# UlanziDock WinUI 3 concepts v1

Twelve image-generated design concepts for a future Windows-only WinUI 3 front end. These PNGs do not implement any application features and do not demonstrate live hardware state.

Open [the gallery](index.html) in a browser to compare dark and light screens, filter by appearance and enlarge each PNG. All images and controls work offline.

The [implementation brief](SPECIFICATION.md) records the six workspaces, Hue/Saturation behavior, proposed slider ranges, architecture and validation boundaries. The [portable manifest](manifest.json) records final image dimensions and SHA-256 hashes. The [complete ZIP](../ulanzi-winui3-concepts-v1.zip) includes every file in this concept folder.

| Function | Dark | Light |
| --- | --- | --- |
| Key editor | [PNG](01-key-editor-dark.png) | [PNG](02-key-editor-light.png) |
| Pages | [PNG](03-pages-dark.png) | [PNG](04-pages-light.png) |
| Applications | [PNG](05-applications-dark.png) | [PNG](06-applications-light.png) |
| Themes | [PNG](07-themes-dark.png) | [PNG](08-themes-light.png) |
| Presets | [PNG](09-presets-dark.png) | [PNG](10-presets-light.png) |
| Device | [PNG](11-device-dark.png) | [PNG](12-device-light.png) |

Hue shifts artwork colors with one rainbow slider; Saturation adjusts color intensity with a second slider. Scope can be a key, a page, or all pages. Reset restores original theme colors. App Light/Dark appearance is separate. A future renderer should preserve legible labels, grayscale inactive-state handling, NEXT PAGE and clock reservations.

The deck preview follows 5 + 5 + 3 square keys and one double-width clock panel. Existing Rust CLI/controller portability and VibeWare artwork identity remain design constraints.

All prompts are in [prompts.md](prompts.md). Images were generated with the built-in image_gen tool.
