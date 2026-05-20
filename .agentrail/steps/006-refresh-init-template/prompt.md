Goal: `cargo run -- init-scene <path>` currently writes `templates/minimal_scene.yaml`, which was authored before the `callouts:` schema existed. That means new users get a stale starter file that the Yew viewer can't fully exercise.

What to do:
- Diff `templates/minimal_scene.yaml` against `scenes/minimal.yaml`. Update the template to match the live schema: include the four objects AND the full `callouts:` block (tooltip, link, svg_billboard, html_panel) with the same anchor / offset conventions.
- Keep comments in the template explaining the anchor schema (`{ object: "..." }` vs `{ world: [x, y, z] }`) — that documentation is the template's main value.
- Add a unit test (in the shared scene crate from step 2) that includes the template via `include_str!`, parses it with `serde_yaml`, and runs `validate()` on it. This locks the template to the schema.

Acceptance:
- `cargo test` includes the new template round-trip test and it passes.
- `cargo run -- init-scene /tmp/out.yaml && cargo run -- validate /tmp/out.yaml` exits 0.
- `diff scenes/minimal.yaml templates/minimal_scene.yaml` shows only intentional differences (e.g., title or comments).

Out of scope: adding new scene types or callout kinds. Schema parity, not schema expansion.
