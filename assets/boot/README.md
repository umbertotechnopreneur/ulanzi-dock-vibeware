# Boot image

`ulanzi-dock-boot-v1.png` is the ImageGen master shown when the controller first connects to the dock. It places ULANZI above DOCK, with the VibeWare logo and name on the right.

The generated master is 1619 × 971 pixels. During a source build, `build.rs` resizes it to a 988 × 592 virtual canvas and cuts it into the dock's native displays: two rows of five 196 × 196 keys, then three 196 × 196 keys and one 400 × 200 panel. It shifts the top band up by 36 pixels and the bottom band down by 36 pixels. The centre band stays fixed. This compensates for the two physical black gaps between the rows. The generated slices stay in Cargo's build folder; this file remains the editable master.

At the first connection in each app run, the controller displays the complete picture for at least two seconds and prepares the remembered page behind it. It then fades the dock brightness to black over one more second, clears the boot picture and restores the saved brightness. The regular page then appears with the 100 ms page animation. The boot display does not accept actions, and its temporary fade does not change the saved brightness.

The original VibeWare logo remains in `assets/brand/vibeware-logo.png`. Image generation used it as a visual reference; this generated composition is not a pixel-identical copy of that source. Preserve the original logo when preparing the final device artwork.

Made with the built-in ImageGen tool. The generation prompt is in `boot-v1.prompt.txt`.

Copyright (c) 2026 Umberto Giacobbi. MIT License: see the repository LICENSE.
