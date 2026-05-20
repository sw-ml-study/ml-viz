Goal: TDD coverage for the pure logic that's currently untested. process.md mandates TDD but `cargo test` finds no tests.

What to do (RED → GREEN per case):
- In the shared scene crate (from step 2), add `#[cfg(test)] mod tests` covering:
  - `validate` rejects: empty objects list; layer_stack with zero layers; network_graph with one or zero nodes; any object with empty/whitespace label.
  - `validate` accepts the bundled `scenes/minimal.yaml`.
  - `resolve_anchor` returns the object's position for `Anchor::Object` when the name matches, `None` when it doesn't, and the literal coords for `Anchor::World`.
  - Serde round-trip: parse `scenes/minimal.yaml` into `Scene`, re-serialize to YAML, parse again, assert equality. Catches `#[serde(default)]` / `#[serde(rename)]` regressions.
- Use `tempfile` only if a test needs a file path; most of these can work on in-memory strings.

Acceptance:
- `cargo test` runs and all new tests pass.
- `cargo clippy --all-targets --all-features -- -D warnings` is clean.

Out of scope: integration tests that shell out to `blender` or `wasm-pack`. Pure unit tests only.
