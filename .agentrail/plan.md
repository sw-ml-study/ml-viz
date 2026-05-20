Unify the scene model and finish the callout pipeline so the CLI, three.js scaffold, and Yew/WASM viewer all share one schema and so callouts flow from `scenes/*.yaml` through to the browser by named anchors. The CLI's `Scene` currently has no `callouts` field, so `gen-three` silently drops them; the Yew viewer already binds callouts but defines its own duplicate types that drift from the CLI's. README explicitly calls out the next step: load `generated/minimal_scene.glb` and attach callout billboards by named anchors.

Steps:
1. bootstrap-agentrail — record the saga itself (this commit).
2. extract-scene-crate — single source of truth for Scene/ObjectSpec/Callout/Anchor, consumed by both CLI and yew-app.
3. tests-for-validate-and-anchors — TDD coverage for validate() edge cases and anchor resolution.
4. refresh-init-template — make templates/minimal_scene.yaml match scenes/minimal.yaml so `init-scene` produces a schema-current file.
5. three-js-callout-parity — bring templates/threejs/main.ts up to callout parity with the Yew viewer (or document Yew as canonical and trim).
6. pre-commit-gate — scripts/pre-commit.sh wired to cargo test + clippy + fmt + markdown-checker + sw-checklist, and prune docs/process.md of `needs-attention` carryover.

Out of scope: GitHub Actions CI, additional scenes, Pi render-job orchestration. These are good follow-ups but separate sagas.
