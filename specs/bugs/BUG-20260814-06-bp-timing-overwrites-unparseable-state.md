# BUG-20260814-06 — bp-timing.sh silently overwrites specs/state.yaml when it cannot parse it

status: logged
severity: high
priority: P1
scope: bigpowers scripts (upstream, not sh-ui source)
discovered: 2026-08-14
discovered_during: plan-work on epic e03

## Symptom

`bash scripts/bp-timing.sh start plan-work` replaced the whole 126-line `specs/state.yaml`
with a 7-line skeleton containing only `metrics.skill_timings`. All decisions, deviations,
handoff, and scope data were gone. `git checkout -- specs/state.yaml` restored them.

## Root cause

Two defects compound:

1. `specs/state.yaml` had been committed with invalid YAML — an unescaped apostrophe inside
   a single-quoted scalar (`cannot be exec'd` in `known_findings`). PyYAML ends the scalar at
   the apostrophe and fails the parse.
2. `bp-timing.sh` start swallows that failure (`except: data = {}`) and then **writes the
   empty skeleton back over the file**. A read-modify-write tool destroyed data it could not
   read, without a non-zero exit or a warning beyond "timing start failed".

## Fix that would work upstream

In `bp-timing.sh`, abort instead of writing when the load fails:

```python
try:
    with open('$STATE_YAML') as f:
        data = yaml.safe_load(f) or {}
except Exception as e:
    sys.stderr.write(f"refusing to touch $STATE_YAML: {e}\n")
    sys.exit(1)
```

and never write the file when `data` came from the fallback.

## Local workaround in sh-ui

- `known_findings` strings now escape apostrophes as `''` (fixed 2026-08-14).
- After any `bp-timing.sh` call, `git diff --stat specs/state.yaml` is checked; an
  unexpected shrink is reverted with `git checkout --`.
