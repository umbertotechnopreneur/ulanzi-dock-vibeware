<!-- Copyright (c) 2026 Umberto Giacobbi | MIT License: see LICENSE -->

# Third-party licenses

My code uses the [MIT License](../LICENSE). Included libraries and fonts keep their own licenses.

- **Slint 1.13.1** draws the app's settings windows on the PC. It is used under the [Slint Royalty-free Desktop, Mobile, and Web Applications License, version 2.0](licenses/slint-royalty-free-2.0.md). Its standard notice is in **About**.
- **Bitstream Vera** supplies the label font. Keep its [font license](../assets/fonts/bitstream-vera-license.txt).
- **Other Rust libraries** have notices in packaged downloads. The file tool collects them from the locked, restored dependencies, including build tools.

Slint uses Winit to open windows and draws controls with its software renderer. The app does not bundle a browser or Python runtime.
