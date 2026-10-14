// story: e02s01
// scenario: SC-e02s01-P2-05
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { RunConfigForm } from "./RunConfigForm";
import type { ScriptEntry } from "./types";

const entry: ScriptEntry = {
  id: "a1",
  path: "/home/user/build.sh",
  config: {
    args: ["a b", "--flag"],
    env: { DEMO_VAR: "hello world" },
    workingDir: "/tmp",
  },
};

describe("RunConfigForm", () => {
  it("prefills every field from the entry configuration", () => {
    render(<RunConfigForm entry={entry} onSave={vi.fn()} error={null} />);

    expect(screen.getByLabelText("Arguments")).toHaveValue("a b\n--flag");
    expect(screen.getByLabelText("Environment")).toHaveValue("DEMO_VAR=hello world");
    expect(screen.getByLabelText("Working directory")).toHaveValue("/tmp");
  });

  it("submits the draft as raw lines", async () => {
    const onSave = vi.fn().mockResolvedValue(undefined);
    render(<RunConfigForm entry={entry} onSave={onSave} error={null} />);

    const args = screen.getByLabelText("Arguments");
    await userEvent.clear(args);
    await userEvent.type(args, "one two\n\nthree");

    await userEvent.click(screen.getByRole("button", { name: /save configuration/i }));

    expect(onSave).toHaveBeenCalledWith({
      argsLines: ["one two", "", "three"],
      envLines: ["DEMO_VAR=hello world"],
      workingDir: "/tmp",
    });
  });

  it("shows the host refusal next to the fields", () => {
    render(
      <RunConfigForm
        entry={entry}
        onSave={vi.fn()}
        error="That working directory does not exist or is not a directory. /nope"
      />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent(
      "That working directory does not exist",
    );
  });
});
