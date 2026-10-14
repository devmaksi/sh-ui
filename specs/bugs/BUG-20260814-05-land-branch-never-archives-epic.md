# BUG-20260814-05 — land-branch.sh never archives a completed epic capsule

status: logged
severity: low
priority: P2
scope: bigpowers scripts (upstream, not sh-ui source)
discovered: 2026-08-14
discovered_during: landing e01s03, the last story of epic e01

## Symptom

`bash scripts/land-branch.sh feat/e01s03-live-output "..."` printed

```
==> Checking for completed epic capsules to archive...
```

and then nothing. `specs/epics/e01-run-and-watch/` stayed where it was, although all three
stories of e01 were `done` in `specs/execution-status.yaml`.

## Root cause

The archival loop reads each story status with:

```bash
STATUS=$(grep "$story_id:" specs/execution-status.yaml | awk '{print $2}')
if [ "$STATUS" != "done" ]; then ALL_DONE=false; break; fi
```

In the bigpowers status file every story id appears **twice**:

```
$ grep -n "e01s01:" specs/execution-status.yaml
3:  e01s01: done          # development_status flat map
41:  e01s01:             # stories block, status on the following line
```

So `STATUS` is two lines — `done` and an empty line — and `[ "$STATUS" != "done" ]` is true.
Every story looks unfinished, so the capsule is never archived. The check silently does nothing
instead of reporting that it skipped.

## Fix that would work upstream

Read the status from the `stories:` block only, or use a YAML reader:

```bash
STATUS=$(awk -v id="$story_id" '
  $1 == id && $2 == ":" { getline; print $2; exit }
' specs/execution-status.yaml)
```

or read `stories.<id>.status` with the same YAML loader the cockpit uses.

## Local workaround in sh-ui

Archive by hand with `git mv` and keep `specs/epics/` for active epics only. `specs/state.yaml`
`handoff.required_reading` points at the archived path.
