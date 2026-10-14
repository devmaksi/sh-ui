// story: e04s02
// scenario: SC-e04s02-P1-02
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { CostReadout } from "./CostReadout";

const gapMetrics = {
  cpuPercent: 50,
  ramBytes: 1048576,
  processes: [],
  vramBytes: null,
  gpuUtilPercent: null,
  gpuSource: "unavailable",
  gpuReason: "no gpu tool found",
};

describe("CostReadout unavailable", () => {
  it("shows a labelled gap with the reason", () => {
    render(<CostReadout metrics={gapMetrics} />);

    expect(screen.getByTestId("cost-gpu-gap")).toHaveTextContent("not measurable");
    expect(screen.getByTestId("cost-gpu-gap")).toHaveTextContent("no gpu tool found");
  });

  it("shows no zero for gpu or vram while cpu and ram stay live", () => {
    render(<CostReadout metrics={gapMetrics} />);

    expect(screen.queryByTestId("cost-gpu")).not.toBeInTheDocument();
    expect(screen.queryByTestId("cost-vram")).not.toBeInTheDocument();
    expect(screen.getByTestId("cost-cpu")).toHaveTextContent("50.0%");
    expect(screen.getByTestId("cost-ram")).toHaveTextContent("1.0 MiB");
  });
});
