// story: e03s02
// scenario: SC-e03s02-P2-04
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { CostReadout } from "./CostReadout";

const treeMetrics = {
  cpuPercent: 300,
  ramBytes: 6291456,
  processes: [
    { pid: 4242, cpuPercent: 100, ramBytes: 2097152 },
    { pid: 4243, cpuPercent: 200, ramBytes: 4194304 },
  ],
};

describe("CostReadout tree", () => {
  it("headline is the summed figure over the tree", () => {
    render(<CostReadout metrics={treeMetrics} />);

    expect(screen.getByTestId("cost-cpu")).toHaveTextContent("300.0%");
    expect(screen.getByTestId("cost-ram")).toHaveTextContent("6.0 MiB");
  });

  it("breakdown appears only after the user opens it", () => {
    render(<CostReadout metrics={treeMetrics} />);

    expect(screen.queryByTestId("cost-process-row")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /details/i }));

    const rows = screen.getAllByTestId("cost-process-row");
    expect(rows).toHaveLength(2);
    expect(rows[0]).toHaveTextContent("pid 4242");
    expect(rows[1]).toHaveTextContent("pid 4243");
  });
});
