# Story Spec Format (20 sections)

bigpowers `plan-work` requires a story spec in countable format with 20 sections and
acceptance criteria in §17. The upstream `countable-story-format.md` is not shipped in the
installed package, so this file is the local source of truth for the section list.

Rules:

- Every story spec uses all 20 sections. No section is dropped. An empty section states why it is empty.
- §12 steps follow `N. <what to do> → verify: <runnable command>`. A step without a runnable command is not a step.
- §17 holds Gherkin acceptance criteria, at least one happy path and one edge case.
- §13 names scenarios `SC-eNNsYY-P{0|1|2|3}-NN`. Test files carry those IDs in comments.
- Words that signal unresolved planning MUST NOT appear in a story spec. `plan-consistency-check.sh` reports them as findings.
- Greenfield stories use the `ADDED` delta tag only.

| § | Section | Purpose |
|---|---------|---------|
| 1 | Identity | Story ID, title, epic, status |
| 2 | Summary | What the story delivers in one paragraph |
| 3 | Classification | type, risk, context, BCPs |
| 4 | Value | Why the story exists for the user |
| 5 | Users and triggers | Who acts, and what starts the behaviour |
| 6 | Domain language | Canonical terms this story introduces |
| 7 | Requirements | Delta-tagged requirements |
| 8 | Behaviour before | What exists today |
| 9 | Behaviour after | What exists when the story ships |
| 10 | Interface and data contracts | IPC commands, events, payload types |
| 11 | Affected modules | Files and crates this touches |
| 12 | Implementation steps | Step → verify pairs |
| 13 | Test strategy | Scenario IDs and test levels |
| 14 | Verification script | Manual UAT the user performs |
| 15 | Out of scope | Explicit exclusions for this story |
| 16 | Risks | What can break and how to detect it early |
| 17 | Acceptance criteria | Gherkin scenarios |
| 18 | Dependencies and prior art | Packages and existing patterns relied on |
| 19 | Error surfaces | How failure is reported to the user |
| 20 | Follow-ups | Work deliberately left for later stories |
