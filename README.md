# ml-viz

Rust CLI scaffold for generating Blender batch Python plus an interactive Rust/Yew/WASM viewer (with three.js via JS glue) from structured ML-visualization scene specs.

## Minimal demo

```bash
cargo run -- init-scene scenes/minimal.yaml
cargo run -- gen-svg --title "Encoder Stack" --out assets/svg/callout_encoder.svg
cargo run -- validate scenes/minimal.yaml
cargo run -- gen-blender scenes/minimal.yaml --out generated/minimal_scene.py
blender -b --python generated/minimal_scene.py
cargo run --example ml-test-scene
```

The Blender step writes `generated/minimal_scene.blend` (with SVG callout curves kept for animation render) and `generated/minimal_scene.glb` (curves excluded so three.js only sees clean PBR geometry).

`cargo run --example ml-test-scene` builds the Yew/WASM crate under `yew-app/` with `wasm-pack` and serves the static HTML, CSS, JS glue, wasm bundle, scene spec, and the .glb on `http://0.0.0.0:9521` so the page is reachable from other devices on the LAN.

## Agent-safe loop

Agents should usually edit `scenes/*.yaml` and templates, then run the Rust CLI. They should avoid hand-editing generated Python except when developing reusable primitives.

## License / copyright

MIT License. See [LICENSE](LICENSE).

Copyright (c) 2026 Michael A. Wright.
