# Threat Model — e05 Protect the single host

Scope: instance lock (e05s01), reap report IPC + UI banner (e05s02). Date: 2026-08-14.

## Surface areas

1. Lock file `<app_data>/app.lock` — path in user-writable app data, opened create+write.
2. `take_reap_report` IPC command — returns pids/pgids of reaped groups to the UI.
3. `ReapBanner` rendering — pid values cross into the DOM.

## Vulnerability analysis

| Category | Finding | Risk | Mitigation |
|---|---|---|---|
| TOCTOU / lock bypass | flock is advisory: a non-sh-ui process could hold or steal the path. | LOW | Accepted: the lock guards sh-ui against sh-ui. Path lives in the app data dir, not a world-writable temp. |
| Symlink swap on lock path | Attacker pre-creates `app.lock` as a symlink elsewhere; host opens with create+write. | LOW | App data dir is user-owned (0700 by default). No cross-user exposure on a single-user desktop. Follow-symlink is acceptable; no privileged write. |
| Lock held after crash | Stale lock blocking starts. | LOW | flock is released by the OS on process death — structurally impossible. Covered by SC-e05s01-P1-03. |
| Info leak via reap report | pids/pgids exposed to own UI over IPC. | NONE | Local IPC to own webview; no network surface; no new listener (DEC: IPC is the only seam). |
| XSS via banner | pid values are u32 from our own host; React escapes text. | NONE | Typed payloads (u32), rendered as text nodes, not HTML. |
| Command injection | None: no shell strings; `flock` via libc call, argv arrays only. | NONE | Existing rule holds. |

## Verdict

No HIGH or MEDIUM findings. Two LOW accepted with rationale above. Safe to proceed to
implementation without additional hardening.
