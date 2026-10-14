// story: e04s01
// scenario: SC-e04s01-P2-04
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { CostReadout } from "./CostReadout";

const gpuMetrics = {
  cpuPercent: 100,
  ramBytes: 2097152,
  processes: [{ pid: 4242, cpuPercent: 100, ramBytes: 2097152 }],
  vramBytes: 367001600,
  gpuUtilPercent: 42,
  gpuSource: "nvidia-smi",
};

describe("CostReadout gpu", () => {
  it("shows attributed vram and device utilization", () => {
    render(<CostReadout metrics={gpuMetrics} />);

    expect(screen.getByTestId("cost-gpu")).toHaveTextContent("42.0%");
    expect(screen.getByTestId("cost-vram")).toHaveTextContent("350.0 MiB");
  });

  it("names the tool that measured", () => {
    render(<CostReadout metrics={gpuMetrics} />);

    expect(screen.getByTestId("cost-gpu-source")).toHaveTextContent("nvidia-smi");
  });
});
