// story: e03s01
import type { RunMetrics } from "./types";

interface CostReadoutProps {
  metrics: RunMetrics | null;
}

function formatRam(ramBytes: number): string {
  return `${(ramBytes / 1048576).toFixed(1)} MiB`;
}

/**
 * Live cost of the selected run. CPU percent is percent of one core (DEC-12): a
 * multi-threaded script may show more than 100. RAM is resident memory.
 */
export function CostReadout({ metrics }: CostReadoutProps) {
  return (
    <div
      data-testid="cost-readout"
      aria-label="Script cost"
      className="flex gap-6 border border-neutral-200 px-3 py-2 text-sm"
    >
      {metrics ? (
        <>
          <span>
            CPU <span data-testid="cost-cpu">{metrics.cpuPercent.toFixed(1)}%</span>
          </span>
          <span>
            RAM <span data-testid="cost-ram">{formatRam(metrics.ramBytes)}</span>
          </span>
        </>
      ) : (
        <span className="text-neutral-500">measuring…</span>
      )}
    </div>
  );
}
