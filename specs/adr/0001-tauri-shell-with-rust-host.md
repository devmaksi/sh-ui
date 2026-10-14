---
status: accepted
date: 2026-08-14
---

# sh-ui runs as a Tauri app: Rust host inside the app, React UI in the webview

sh-ui must launch shell scripts, supervise them, read their output, and measure their
process tree on the local machine. That needs real process control and `/proc` access. A
browser-only app cannot do it, and a separate daemon the UI talks to over HTTP would open a
localhost port, need its own lifecycle, and need its own auth story. We chose Tauri: the
Rust side of the app owns spawning, supervision, output capture, and metric reading. The
React + Vite side owns presentation only. They communicate over Tauri IPC commands and
events, not a network socket.

## Considered options

- **Local web app served by a Node daemon.** Rejected. It adds a listening port, a second
  process to keep alive, and an authentication surface for a single local user.
- **Electron.** Rejected. It solves the same problem with a much heavier runtime, and the
  process-control code would still sit in Node rather than closer to the OS.
- **GTK in Python or C.** Rejected. Outside the chosen stack, and the UI work would be
  slower than the React work the project is already set up for.

## Consequences

- The project is two-language: TypeScript for the UI, Rust for the host. Preflight runs both.
- Rust is not a general-purpose backend here. It is the process host. UI logic MUST NOT move into Rust.
- Linux build dependencies are now part of the environment: `webkit2gtk-4.1`, `libgtk-3-dev`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`.
- Packaging, tray, and window behaviour are Tauri concerns, not React concerns.
- Verified available on the development machine: cargo 1.98.0, node 22.23.2, webkit2gtk 4.1.
