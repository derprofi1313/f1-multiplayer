# T### <question>

- Requested by: <agent>, <date>
- Status: draft
- Session: (filled by the lead when batched)

## Question
<question>
One question only. If it splits into two, file two tests.

## Hypothesis
What you expect and why, citing KB items (`functions.tsv` names, struct fields, signature ids).

## Promotes
KB items that move to L3 if this passes (and which demote if it fails):
- `Name` (functions/globals), `Struct.field`, `sig.id`

## Build
- exe: F1_25.exe sha256 d873a1c8d2044cef… (must equal `re/db/signatures.json` build)
- loader: `native/dist/version.dll` md5 …
- plugins: `swofty/mods/core/swofty_core.dll` md5 …, `swofty/mods/swofty_trace.dll` md5 …
- data files (if any): path + md5
- install: what goes where under the game folder (only `version.dll` and `swofty/`)

## Trace config
Authoring config for `re/tools/trace_build.py` (hook ids by signature id), or a path to `re/tests/T###.trace.json`. Trace-only hooks need their targets at L1+ and passing `sigcheck.py`.

## User steps
1. Numbered, concrete, minimal. No save edits, no online.

## Expected log lines
Regexes against `swofty/trace.log` / `swofty/loader.log`, each with what it proves:
- `regex` → proves …
Pass rule: … Fail rule: …

## Result
- Run id: (lead) — logs filed under `re/traces/T###/<run>/`
- Verdict: pass / fail / inconclusive
- KB changes made: (rows promoted or demoted, with `trace:T###/<run>:<line>` tokens)
