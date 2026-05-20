Goal: bring the three.js scaffold to parity with the Yew viewer, OR explicitly retire it in favor of Yew. README says: "the next step is to load `generated/minimal_scene.glb` and attach callout billboards by named anchors" — currently only the Yew viewer does this; `templates/threejs/main.ts` recreates the scene directly and ignores `.glb` + callouts.

Decision branch (pick one early, document in the commit):

(A) Parity path:
- Update `templates/threejs/main.ts` to load `minimal_scene.glb` (the `gen-three` command already copies it to `public/`).
- Read `public/scene.json` (now with callouts after step 2), and for each callout resolve its anchor by name against the loaded glTF scene (walk `gltf.scene` for an `Object3D` whose `name` matches the object name). Fall back to the position in `scene.json` if not found.
- Render the four callout kinds (`tooltip`, `link`, `svg_billboard`, `html_panel`) as HTML elements positioned over the canvas — the Yew app's `glue.js` is a good reference.
- A `README.md` snippet showing how to run the resulting Vite demo end-to-end.

(B) Retire path:
- Delete `templates/threejs/` and the `GenThree` CLI subcommand.
- Remove the `web-example/` scaffold (or move it under `.gitignored` generated output).
- Update README + docs/phases.md to make Yew the single canonical viewer, with a one-paragraph rationale in the commit message.

Pick (A) if you want both viewers as living examples; pick (B) if you'd rather not maintain two. Either choice resolves the README's outstanding "next step".

Acceptance for (A):
- `cargo run -- gen-three scenes/minimal.yaml --out-dir web-example && cd web-example && npm install && npm run dev` loads the glb and shows all four callout kinds bound to their anchor objects in a browser.

Acceptance for (B):
- `cargo run -- gen-three ...` no longer exists; README and docs/phases.md are consistent; `cargo build` passes.
