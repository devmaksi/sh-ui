# e04s02 — Say what cannot be measured

## 1. Identity

Story ID: e04s02. Title: Say what cannot be measured. Epic: e04-gpu-visibility. Status:
planned.

## 2. Summary

When no GPU tool answers, the readout says plainly that GPU and VRAM are not measurable on
this machine and names the reason. Zero is never shown for an unknown. This protects the
credibility of every number the app does show.

## 3. Classification

type: feat · risk: P1 · context: domain · bcps: 3

## 4. Value

This is the epic's value test: a user on a machine without GPU tooling sees "not measurable
here", never a zero that looks like a measurement.

## 5. Users and triggers

Any run on a machine where the probe found no GPU source, or where the source stopped
answering mid-run.

## 6. Domain language

**Unavailable metric** — a metric the machine cannot measure, carried as null plus a reason
string. Distinct from a measured zero. **Probe reason** — the plain-words cause the UI shows:
"no gpu tool found", "intel_gpu_top needs permissions this app does not have", and so on.

## 7. Requirements

#### MODIFIED: Unmeasurable metrics are an explicit state
Before: the metrics payload carries nullable `vramBytes` and `gpuUtilPercent` plus a
`gpuSource` tag. After: it also carries `gpuReason` — a plain-words explanation shown
whenever the source is unavailable or a query stopped answering. Nulls stay null; the host
never substitutes zero.

#### ADDED: The gap is rendered as a labelled gap
When the source is unavailable the readout shows "GPU: not measurable" with the reason. It
shows no zero, no dash-only ambiguity, and keeps CPU and RAM figures live beside it.

## 8. Behaviour before

e04s01 sends nulls when the probe fails, but the readout has no defined rendering for them.

## 9. Behaviour after

An unmeasurable GPU is a labelled, explained gap in the readout. CPU and RAM are unaffected.

## 10. Interface and data contracts

`run://metrics` gains `gpuReason: String | null`. Host: `GpuSource::Unavailable` carries the
reason text; a probe that fails for permissions says so. UI: `RunMetrics.gpuSource` and
`gpuReason`; the readout renders the unavailable branch.

## 11. Affected modules

`src-tauri/src/gpu.rs` — reasons on Unavailable. `src-tauri/src/runs.rs` — payload field.
`src/features/runs/CostReadout.tsx` — unavailable branch.

## 12. Implementation steps

1. Unavailable carries a reason through the snapshot and payload; null is never zero → verify: `cargo test --manifest-path src-tauri/Cargo.toml metric_unavailable`
2. The readout renders the labelled gap with the reason and keeps CPU and RAM live → verify: `npm run test -- CostReadoutUnavailable`
3. Manual verification with GPU tooling hidden from PATH → verify: the Verification Script below, recorded in `specs/verifications/`

## 13. Test strategy

| Scenario ID | Level | Asserts |
|---|---|---|
| SC-e04s02-P0-01 | U-R | An unavailable source serializes as null fields plus a reason; no field is zero-filled |
| SC-e04s02-P1-02 | U-T | The readout shows "not measurable" with the reason and keeps CPU and RAM |
| SC-e04s02-P1-M1 | M | With tools hidden from PATH the app shows CPU and RAM and names the GPU gap |

## 14. Verification Script

1. Find where the vendor tool lives: `which nvidia-smi`.
2. Start the app with a PATH that excludes that directory (for example
   `PATH=/usr/local/bin:/usr/bin env PATH="$(echo $PATH | sed 's\|/usr/local/bin:\|\')" npm run tauri dev`
   or move the user session to a shell without it).
3. Start a script.
4. Confirm CPU and RAM still show and update.
5. Confirm GPU and VRAM show as "not measurable" with a reason, not as 0.

## 15. Out of scope

Installing missing vendor tools, prompting for root, per-metric partial reasons beyond the
probe failure text, historical availability.

## 16. Risks

- A zero leaking in as a default would masquerade as a measurement. The payload test asserts
  nulls survive serialization; the UI test asserts no zero appears in the unavailable branch.
- A reason string that is a raw tool stderr could leak noise to the UI. Reasons are a closed
  set written by the host, not raw stderr.

## 17. Acceptance criteria

```gherkin
# SC-e04s02-P0-01
Scenario: Unknown is never zero
  Given a run whose gpu source is unavailable
  When the metrics event serializes
  Then vram and gpu utilization are null
  And a reason string is present

# SC-e04s02-P1-02
Scenario: The gap is honest
  Given a metrics event with an unavailable gpu source
  When the runs view renders
  Then it shows "not measurable" with the reason
  And no zero appears for gpu or vram
  And the cpu and ram figures still render
```

## 18. Dependencies and prior art

Builds on e04s01's probe. No new packages. Prior art: the scope constraint "Missing tooling
is reported as unavailable" (`specs/product/SCOPE_LATEST.yaml`).

## 19. Error surfaces

Probe reasons are one of: "no gpu tool found", "gpu tool failed to run", "gpu tool needs
permissions this app does not have". Anything a vendor tool prints on stderr stays in the
host log, not the UI.

## 20. Follow-ups

None for this epic. GPU per-process time stays out of scope until a vendor makes it
reliable.
