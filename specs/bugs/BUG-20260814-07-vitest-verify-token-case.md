# BUG-20260814-07 — vitest verify tokens in task YAMLs never matched the test files

status: logged
severity: low
priority: P2
scope: bigpowers planning data (slice-tasks output), vitest 5.0.4 behavior
discovered: 2026-08-14
discovered_during: e03s01 task 5 (`npm run test -- cost-readout` found no test files)

## Symptom

`npm run test -- cost-readout` exits 1 with "No test files found" even though
`src/features/runs/CostReadout.test.tsx` exists and passes. The same is true for the
e01-era tokens: `npm run test -- output-panel` also finds nothing.

## Root cause

Vitest 5 CLI filters are case-sensitive substrings of the file path. Kebab-case tokens
(`cost-readout`, `output-panel`) never match PascalCase file names (`CostReadout.test.tsx`,
`OutputPanel.test.tsx`). `slice-tasks` wrote verify commands in kebab-case from story titles.

The archived e01 task YAMLs carry the same broken tokens; those tasks were verified through
the full `npm run test` suite, so the defect stayed hidden.

## Fix applied locally

Active e03 task YAMLs now use the exact PascalCase file stem as the filter token
(`CostReadout`, `CostReadoutTree`). Archived e01 files are left as historical record.

## Fix that would work upstream

`slice-tasks` / `plan-work` templates should state: vitest verify tokens must be the exact
case-sensitive file stem of the intended test file.
