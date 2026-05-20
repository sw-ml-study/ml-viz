Goal: stratify the cone backdrop into three visual bands so the silhouette reads as mountains > hills > trees from back to front. Live demo feedback while reviewing the Yew viewer.

What to do (Blender template only — no schema or code changes):
- templates/blender/scene.py.tera: replace the single 44-peak cone loop with three depth-banded loops, in this back-to-front order:
  * Back (~y=22): largest cones (tallest height, biggest radius). Darkest brown (browns[0] = mountain-umber). These hug the horizon and are partly clipped by the plane edge.
  * Middle (~y=18): medium cones. A clay-brown shade halfway between the dark mountains and the sand-colored ground plane (browns[2] = mountain-clay fits).
  * Front (~y=14): smallest cones — "trees". Green (existing accent-green material).
- Keep vertices=3 throughout so the silhouette stays triangular.
- Regenerate generated/minimal_scene.py via `cargo run -- gen-blender ...` and re-bake generated/minimal_scene.glb via `blender -b --python`.

Acceptance:
- cargo build/clippy/fmt/test all clean (no Rust changes).
- New .glb loads in the viewer (refresh http://localhost:9521); back row dark/large, middle clay-brown/medium, front green/small.

Out of scope: changing the ground plane size, the row of demo objects, callout positions, or any non-template code.
