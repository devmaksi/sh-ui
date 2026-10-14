// story: e01s03
// scenario: SC-e01s03-P1-07
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { OutputPanel } from "./OutputPanel";

describe("OutputPanel", () => {
  it("offers no way to send anything to the run", () => {
    render(
      <OutputPanel
        lines={[{ id: 1, stream: "stdout", chunk: "out-1" }]}
        droppedCount={0}
      />,
    );

    expect(screen.queryByRole("textbox")).toBeNull();
    expect(screen.queryByRole("combobox")).toBeNull();
    expect(screen.queryByRole("button")).toBeNull();
    expect(document.querySelector("input, textarea, [contenteditable]")).toBeNull();
  });
});
