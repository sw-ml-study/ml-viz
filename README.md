# blender-ml-viz

Rust CLI scaffold for generating Blender batch Python and a three.js web example from structured ML visualization scene specs.

## Minimal demo

```bash
cargo run -- init-scene scenes/minimal.yaml
cargo run -- gen-svg --title "Encoder Stack" --out assets/svg/callout_encoder.svg
cargo run -- validate scenes/minimal.yaml
cargo run -- gen-blender scenes/minimal.yaml --out generated/minimal_scene.py
blender -b --python generated/minimal_scene.py
cargo run -- gen-three scenes/minimal.yaml --out-dir web-example
cd web-example && npm install && npm run dev
```

The Blender script exports both `generated/minimal_scene.blend` and `generated/minimal_scene.glb`.

## Agent-safe loop

Agents should usually edit `scenes/*.yaml` and templates, then run the Rust CLI. They should avoid hand-editing generated Python except when developing reusable primitives.
