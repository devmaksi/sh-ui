# sh-ui

**A Linux desktop app that manages a registry of your shell scripts** — launch them with
your own arguments, environment, and working directory; watch their output live; see what
they cost; stop them with one press; and never leave a stray process behind.

![sh-ui: a live run with output and cost readout](docs/screenshots/sh-ui-live-run.png)

## What it does

- **Script registry.** Register any executable file. Per script you can store argv items,
  `KEY=VALUE` environment pairs, and a working directory. The registry survives restarts.
- **Live output.** stdout and stderr stream into the UI as the script runs, with dropped
  lines counted instead of the webview growing without bound.
- **Live cost.** CPU percent (per core, top-style) and resident memory for the script and
  its whole process tree, refreshed every second.
- **GPU and VRAM, honestly.** When the machine can measure them (nvidia-smi, amd-smi,
  rocm-smi, intel_gpu_top), per-run VRAM attribution and device utilization are shown.
  When it cannot, the app says exactly why instead of guessing.
- **One-press stop.** A run is spawned as its own process group; Stop sends SIGTERM to the
  whole group, then SIGKILL after a two-second grace. Forked children die with the parent.
- **Honest end status.** Every finished run reports `exited`, `failed`, or `stopped` —
  derived from the real exit code and signal, not guessed.
- **Leave nothing running.** Closing the app stops every live run. If the app crashed, the
  next start reaps the orphan groups it left behind — verified by process start time, so a
  recycled pid is never killed.

## How it works

A Tauri (Rust) host owns the processes: it spawns each script directly as a process-group
leader (`setsid`), supervises it, caps output at 8 KiB per event, and reads `/proc`
through the `procfs` crate for metrics. The React UI talks to the host over Tauri IPC
only — no network port, no second daemon. The registry and the live-run recovery record
are plain JSON files in the application data directory.

```
src-tauri/   Rust host: supervisor, metrics, GPU probes, registry, recovery
src/         React UI: features/registry, features/runs, shared components
```

## Install

Linux only (the host reads `/proc`).

From a release bundle:

```sh
sudo dpkg -i sh-ui_0.1.0_amd64.deb    # deb
sudo rpm -i sh-ui-0.1.0-1.x86_64.rpm  # rpm
./sh-ui_0.1.0_amd64.AppImage          # AppImage
```

## Build from source

Requirements: Node 22, Rust, a Linux desktop toolchain (see the
[Tauri prerequisites](https://tauri.app/start/prerequisites/)).

```sh
npm install
npm run tauri dev      # development
npm run tauri build    # release bundles -> src-tauri/target/release/bundle/
```

## Test and lint

```sh
npm run lint           # ESLint + tsc --noEmit + clippy -D warnings
npm run test           # Vitest (UI) + cargo test (host)
npm run preflight      # the full gate: lint && test && cargo test && build
```

## GPU measurement sources

| Source | Condition |
|---|---|
| `nvidia-smi` | per-process VRAM via `--query-compute-apps` |
| `amd-smi` / `rocm-smi` | process and device queries where available |
| `intel_gpu_top` | device utilization only, and only when it runs without root |
| none of the above | the app reports an explicit unavailable state with the reason |

## Version

0.1.0 — see `specs/release-plan.yaml` for the release contents and `specs/` for the full
specification, architecture decisions, and verification evidence.
