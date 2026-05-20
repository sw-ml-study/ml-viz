Goal: eliminate the duplicate Scene model. Right now `src/main.rs` and `yew-app/src/lib.rs` each define their own `Scene` / `ObjectSpec` / (in Yew only) `Callout` / `Anchor`, and they have drifted — the CLI's `Scene` has no `callouts` field, so `gen-three` writes a callout-less `scene.json`.

What to do:
- Extract a shared library crate (`src/lib.rs` or a new workspace member like `scene/`) holding `Scene`, `SceneMeta`, `ObjectSpec`, `Callout`, `Anchor`, and `resolve_anchor`. Pick whichever crate layout fits the existing `[workspace]` in Cargo.toml.
- Make both `src/main.rs` (binary) and `yew-app` depend on it. Delete the duplicate definitions.
- Add `callouts: Vec<Callout>` (with `#[serde(default)]`) to the shared `Scene`. Confirm `gen_three` now serializes them into `public/scene.json`.
- Keep `ObjectSpec` as the tagged enum from the CLI side — it carries the per-type fields (`layers`, `nodes`, `svg`). The Yew side's flat struct loses information; converge on the richer one and adjust Yew's UI to read it.

Acceptance:
- `cargo build --workspace` and `cargo build -p yew-app --target wasm32-unknown-unknown` (or whatever the existing build script uses) both succeed.
- `cargo run -- gen-three scenes/minimal.yaml --out-dir web-example` produces a `scene.json` whose `callouts` array matches `scenes/minimal.yaml`.
- The Yew app still renders the HUD and tooltips correctly (a quick `cargo run --example ml-test-scene` and a browser load is enough).

Out of scope: changes to the three.js scaffold beyond what the new shared types force. That's step 5.
