// story: e03s01 e03s02
import { useState } from "react";
import type { RunMetrics } from "./types";

interface CostReadoutProps {
  metrics: RunMetrics | null;
}

function formatRam(ramBytes: number): string {
  return `${(ramBytes / 1048576).toFixed(1)} MiB`;
}

/**
 * Live cost of the selected run. The headline is the sum over the whole process tree:
 * CPU percent of one core (DEC-12) and summed resident bytes (DEC-13). The per-process
 * breakdown sits behind an explicit control so the sum leads and the detail waits.
 */
export function CostReadout({ metrics }: CostReadoutProps) {
  const [open, setOpen] = useState(false);

  if (!metrics) {
    return (
      <div
        data-testid="cost-readout"
        aria-label="Script cost"
        className="border border-neutral-200 px-3 py-2 text-sm"
      >
        <span className="text-neutral-500">measuring…</span>
      </div>
    );
  }

  return (
    <div
      data-testid="cost-readout"
      aria-label="Script cost"
      className="border border-neutral-200 px-3 py-2 text-sm"
    >
      <div className="flex items-center gap-6">
        <span>
          CPU <span data-testid="cost-cpu">{metrics.cpuPercent.toFixed(1)}%</span>
        </span>
        <span>
          RAM <span data-testid="cost-ram">{formatRam(metrics.ramBytes)}</span>
        </span>
        {metrics.vramBytes !== null ? (
          <span>
            VRAM <span data-testid="cost-vram">{formatRam(metrics.vramBytes)}</span>
          </span>
        ) : null}
        {metrics.gpuUtilPercent !== null ? (
          <span>
            GPU{" "}
            <span data-testid="cost-gpu">{metrics.gpuUtilPercent.toFixed(1)}%</span>
            <span data-testid="cost-gpu-source" className="text-neutral-500">
              {" "}
              (device, {metrics.gpuSource})
            </span>
          </span>
        ) : (
          <span data-testid="cost-gpu-gap" className="text-neutral-500">
            GPU: not measurable
            {metrics.gpuReason ? ` — ${metrics.gpuReason}` : ""}
          </span>
        )}
        <button
          type="button"
          onClick={() => setOpen((current) => !current)}
          className="ml-auto border border-neutral-300 px-2 py-0.5 text-xs"
        >
          Details
        </button>
      </div>

      {open ? (
        <ul className="mt-2 font-mono text-xs">
          {metrics.processes.map((process) => (
            <li
              key={process.pid}
              data-testid="cost-process-row"
              className="flex gap-4"
            >
              <span>pid {process.pid}</span>
              <span>{process.cpuPercent.toFixed(1)}%</span>
              <span>{formatRam(process.ramBytes)}</span>
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}
