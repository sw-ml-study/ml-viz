Goal: docs/process.md mandates a multi-step pre-commit gate (cargo test, clippy -D warnings, fmt, markdown-checker, sw-checklist) but nothing actually runs it, and `docs/process.md` still has stale `needs-attention` carryover (it references `needs_attention.db`, `gather.sh`, and a project name that isn't this one).

What to do:
- Create `scripts/pre-commit.sh` that runs, in order:
  1. `cargo test --workspace`
  2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  3. `cargo fmt --all -- --check`
  4. `markdown-checker -f "**/*.md"` (if installed; otherwise warn and skip — don't hard-fail on a missing tool)
  5. `sw-checklist` (same: warn-and-skip if not on PATH)
  Use `set -euo pipefail`. Each step prints a clear header. Exit non-zero on the first failure.
- Wire it from `.git/hooks/pre-commit`? No — leave that as a developer choice. Just document it in README under a new "Pre-commit" section.
- Edit `docs/process.md`:
  - Remove or rewrite any reference to `needs-attention`, `needs_attention.db`, `gather.sh`, `wasm-pack` setup steps that don't match this repo, the "build needs-attention-v0.1.0-macos.tar.gz" release block, and the static/ directory.
  - Keep the TDD red/green/refactor section and the pre-commit sequence, but rephrase so they reference `scripts/pre-commit.sh` and ml-viz's actual layout (`src/`, `yew-app/`, `templates/`, `scenes/`, `generated/`).
  - Drop the dead "needs-attention" Database Migrations / Database Corruption sections entirely.

Acceptance:
- `bash scripts/pre-commit.sh` runs cleanly on a clean tree.
- `markdown-checker -f "docs/process.md"` passes.
- `grep -ri needs-attention docs/ README.md` returns nothing.
- This step is `--done`, closing the saga.
