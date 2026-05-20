Goal: tighten the Yew viewer's default camera, scene layout, callout sizing, and HUD now that it's the canonical viewer (step 003 retired the alternative). Captures live demo-tuning feedback collected while reviewing on http://localhost:9521.

What to do:
- scenes/minimal.yaml: put all four demo objects on a single horizontal line (same y, same z; spread along x). Shrink svg_billboard width and tighten callout offsets so each callout sits close to its anchor.
- templates/blender/scene.py.tera: enlarge the ground plane (much wider) and spread/push back the mountain backdrop so it doesn't crowd the row.
- Regenerate generated/minimal_scene.py and re-bake generated/minimal_scene.glb via blender (the .glb is gitignored; just verify it loads).
- yew-app/static/glue.js: set the default camera to a low, front-on framing of the row. Factor camera/target init values into named constants and expose `window.homeView()` that resets to them.
- yew-app/src/lib.rs: extern home_view, add a "Home view" button at the top of the HUD that calls it.
- yew-app/static/style.css: HUD aside narrower, taller (max-height fills the viewport), left-aligned content, room for the new home button. Bump #hud z-index above #callouts-layer so the title doesn't get covered.

Acceptance:
- cargo build --workspace, clippy -D warnings, fmt --check all clean.
- wasm-pack rebuild succeeds (`cargo run --example ml-test-scene` rebuilds yew-app automatically).
- Browser at :9521 shows the four demo objects on a horizontal line in front of a wide salt flat with a far backdrop; callouts sit close to their objects; HUD is narrower/taller with a working Home view button.

Out of scope: testing infrastructure (that's the next step) and template-refresh (later step).
