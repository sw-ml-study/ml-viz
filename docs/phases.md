# Pipeline phases

## 1. Agent → scene spec

The planning agent converts a concept prompt into `scenes/*.yaml`. This is where labels, object positions, object types, and educational sequencing are chosen.

## 2. Rust → generated assets and scripts

The Rust CLI validates the scene, generates NanoBanana-style SVG callouts, emits Blender Python from templates, and emits a three.js/Vite example.

## 3. Agent + Pi → Blender batch execution

A Raspberry Pi or other batch node can run orchestration commands and dispatch Blender jobs to a GPU/CPU render host:

```bash
blender -b --python generated/minimal_scene.py
```

Outputs:

- `.blend` for continued Blender editing
- `.glb` for web and three.js
- rendered frames or movie files when render commands are enabled

## 4. Blender out → model + Rust/Yew/three.js

The `.glb` model becomes the shared scene artifact. A Rust/Yew app can wrap three.js bindings or call JavaScript interop to load the model, fly the camera, and keep SVG/canvas callouts facing the camera.

## 5. Runnable web example

`cargo run -- gen-three scenes/minimal.yaml --out-dir web-example` creates a Vite/three.js demo. The scaffold currently recreates the minimal scene directly; the next step is to load `generated/minimal_scene.glb` and attach callout billboards by named anchors.
