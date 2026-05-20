# Pipeline phases

## 1. Agent -> scene spec

The planning agent converts a concept prompt into `scenes/*.yaml`. This is where labels, object positions, object types, and educational sequencing are chosen.

## 2. Rust -> generated assets and scripts

The Rust CLI (`ml-viz`) validates the scene, generates NanoBanana-style SVG callouts, and emits Blender Python from templates. There is no separate web-side generator: the interactive viewer is a single Rust/Yew/WASM crate that reads the same scene spec at runtime.

## 3. Agent + Pi -> Blender batch execution

A Raspberry Pi or other batch node can run orchestration commands and dispatch Blender jobs to a GPU/CPU render host:

```bash
blender -b --python generated/minimal_scene.py
```

Outputs:

- `.blend` for continued Blender editing
- `.glb` for the Yew viewer
- rendered frames or movie files when render commands are enabled

## 4. Blender out -> model + Rust/Yew viewer

The `.glb` model becomes the shared scene artifact. The Yew app under `yew-app/` loads it via three.js (through a thin JS glue layer), flies the camera on demand, and renders DOM callouts (tooltips, links, SVG billboards, HTML panels) positioned over the canvas via named anchors against the scene spec.

## 5. Runnable demo

```bash
cargo run --example ml-test-scene
```

builds the Yew crate with `wasm-pack` and serves the page on `http://0.0.0.0:9521`. The page resolves `scene.json` (the YAML spec serialized to JSON) and `minimal_scene.glb` (the Blender export) and binds each callout to its anchor object by name.
