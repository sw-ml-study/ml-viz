Goal: record the agentrail saga itself in git so future sessions inherit a tracked record.

What to do:
- Verify `.agentrail/` exists with `saga.toml`, `plan.md`, and one step folder per saga entry.
- Stage the entire `.agentrail/` tree (and `AGENTS.md` if not already tracked).
- Commit with a message like "chore: bootstrap agentrail saga for unify-scene-and-callout-pipeline".
- Then `agentrail complete --summary "..." --reward 1 --actions "..."`.

Out of scope: any code changes. This step is administrative; the real work starts in step 2.

Acceptance:
- `git log -1` shows the bootstrap commit and `git ls-files .agentrail/` is non-empty.
- `agentrail status` shows step 1 completed.
