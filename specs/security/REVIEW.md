# Security Review — e02s04 (feat/e02s04-shutdown-cleanup)

Diff reviewed: `main..HEAD` (recovery.rs new; supervisor.rs shutdown/ExitWatch/poller-join;
lib.rs reap at setup; tests). Reviewed 2026-10-15 as part of verify-work step 5 (risk P1).

## Findings

| # | Severity | Confidence | Finding | Disposition |
|---|----------|------------|---------|-------------|
| 1 | LOW | 6 | `reap_stale` kills recorded process groups. A tampered `live-runs.json` could make the host kill arbitrary pids. | Accepted. The file lives in the user-owned app data dir; an attacker who can write there can already signal the user's processes. The start-time match (procfs `Stat::starttime`) blocks the accidental pid-reuse case, which is the realistic threat. |
| 2 | LOW | 5 | TOCTOU: a pid could be recycled between the start-time check and `killpg`. | Accepted. Microsecond window; same discipline as systemd's start-time identity check. A mismatched record is dropped without killing (unit-tested). |
| 3 | INFO | — | `load_records` deserializes JSON with serde into a typed struct; corrupt file loads as empty. | Safe. No dynamic deserialization, no code path from file content beyond three integers. |
| 4 | INFO | — | Atomic write discipline (temp + rename) reused from `Registry::save`. | No partial-write corruption of the record file. |
| 5 | INFO | — | No new IPC commands, no network surface, no shell string built from user input (argv arrays unchanged). | Unchanged contract. |

## Verdict

No HIGH findings with confidence ≥ 8. Gate: **PASS**.
