// story: e01s03
// scenario: SC-e01s03-P1-06
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { OutputPanel } from "./OutputPanel";
import type { OutputLine } from "./types";

const lines: OutputLine[] = [
  { id: 1, stream: "stdout", chunk: "out-1" },
  { id: 2, stream: "stderr", chunk: "warn-1" },
  { id: 3, stream: "stdout", chunk: "out-2" },
];

describe("OutputPanel", () => {
  it("shows chunks in arrival order", () => {
    render(<OutputPanel lines={lines} droppedCount={0} />);

    const rows = screen.getAllByTestId("output-line");
    expect(rows.map((row) => row.textContent)).toEqual(["out-1", "warn-1", "out-2"]);
  });

  it("marks stderr so it is never mistaken for normal output", () => {
    render(<OutputPanel lines={lines} droppedCount={0} />);

    const rows = screen.getAllByTestId("output-line");
    expect(rows[1]).toHaveAttribute("data-stream", "stderr");
    expect(rows[0]).toHaveAttribute("data-stream", "stdout");
  });

  it("says how many lines the ring buffer dropped", () => {
    render(<OutputPanel lines={lines} droppedCount={37} />);

    expect(screen.getByText(/37 lines hidden/i)).toBeInTheDocument();
  });

  it("says the run has produced nothing yet", () => {
    render(<OutputPanel lines={[]} droppedCount={0} />);

    expect(screen.getByText(/no output yet/i)).toBeInTheDocument();
  });
});
