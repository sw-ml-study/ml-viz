#!/usr/bin/env bash
set -euo pipefail
cargo run -- init-scene scenes/minimal.yaml
cargo run -- gen-svg --title "Encoder Stack" --out assets/svg/callout_encoder.svg
cargo run -- validate scenes/minimal.yaml
cargo run -- gen-blender scenes/minimal.yaml --out generated/minimal_scene.py
# blender -b --python generated/minimal_scene.py
# cargo run --example ml-test-scene
