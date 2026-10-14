# BUG-20260814-04 — bigpowers dashboard web client returns 404 because its own dependencies are not installed

status: logged
severity: low
priority: P2
scope: bigpowers dashboard (upstream)
discovered: 2026-08-14
discovered_during: opening the project dashboard for sh-ui

## Symptom

`node dashboard/bin/dashboard.js --web --project <sh-ui>` starts and answers the API:

```
GET /api/state -> 200
GET /          -> 404 NotFoundError: Not Found
```

The HTML client never loads, so the browser shows nothing.

## Root cause

`dashboard/package.json` declares `express 4.19.0`, but `dashboard/node_modules` does not exist
in the installed package. Node then resolves express from a neighbouring install and gets
express 5.3.0. Under express 5, `res.sendFile(<absolute path>)` fails through `send` with a 404
even though the file exists.

Minimal reproduction, outside this project:

```
sending /…/bigpowers/dashboard/src/web/client.html true
NotFoundError: Not Found
    at createHttpError (/…/node_modules/send/index.js:861:12)
    at SendStream.error (/…/node_modules/send/index.js:168:31)
    at sendfile (/…/node_modules/express/lib/response.js:1011:8)
```

## Impact

The dashboard API is usable, the dashboard UI is not. An agent reports "the dashboard does not
work" when only the client route is broken.

## Proposed fix (upstream)

1. Install dashboard dependencies as part of `bigpowers setup`, or ship `dashboard/node_modules`.
2. Pin the express major in code, or replace `res.sendFile` with a read-and-send that works on
   both express 4 and 5.

## Local resolution

`cd <bigpowers>/dashboard && npm install` was run on 2026-08-14. `dashboard/node_modules/express`
is now 4.19.0, the version the package declares. After a restart, `GET /` returns 200 and
`GET /api/state` returns 200.

sh-ui now exposes it as project commands: `npm run dashboard` and `npm run dashboard:stop`,
through `tools/dashboard.sh`, which resolves the bigpowers install path from the `scripts`
symlink instead of hardcoding a path.

## Working alternative

The `visual-dashboard` cockpit server has no third-party dependencies and serves both the API
and the HTML view:

```
bash skills/visual-dashboard/scripts/start-server.sh --project-dir <project> --background
GET /api/status?projectDir=<project>
GET /cockpit.html?projectDir=<project>
```

## Local note on YAML style

`skills/visual-dashboard/scripts/read-specs-status.cjs` reads YAML with a line parser that does
not strip trailing comments. An inline comment after a value became part of the value:

```
version: "0.1.0"          # mirror label only
-> version = 0.1.0"          # mirror label only …
```

sh-ui now keeps comments on their own line in `specs/release-plan.yaml`. That is a style change
on our side, not a fix for the parser.
