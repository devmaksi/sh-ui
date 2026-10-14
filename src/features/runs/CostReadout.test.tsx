// story: e03s01
// scenario: SC-e03s01-P2-05
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { CostReadout } from "./CostReadout";

describe("CostReadout", () => {
  it("shows cpu percent and resident ram from a metrics event", () => {
    render(<CostReadout metrics={{ cpuPercent: 42.5, ramBytes: 1048576, processes: [], vramBytes: null, gpuUtilPercent: null, gpuSource: "unavailable", gpuReason: "no gpu tool found" }} />);

    expect(screen.getByTestId("cost-cpu")).toHaveTextContent("42.5%");
    expect(screen.getByTestId("cost-ram")).toHaveTextContent("1.0 MiB");
  });

  it("says measuring before the first metrics event arrives", () => {
    render(<CostReadout metrics={null} />);

    expect(screen.getByTestId("cost-readout")).toHaveTextContent("measuring");
  });
});
