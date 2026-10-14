// story: e01s02
// scenario: SC-e01s02-P2-10
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { AddScriptForm } from "./AddScriptForm";

describe("AddScriptForm", () => {
  it("puts the picked path in the field and keeps it editable", async () => {
    const picked = "/home/user/build.sh";
    const pickPath = vi.fn(async () => picked);
    const onAdd = vi.fn(async () => undefined);

    render(<AddScriptForm onAdd={onAdd} pickPath={pickPath} />);

    await userEvent.click(screen.getByRole("button", { name: "Choose file" }));

    const field = screen.getByRole("textbox");
    expect(field).toHaveValue(picked);

    await userEvent.clear(field);
    await userEvent.type(field, "/home/user/other.sh");
    await userEvent.click(screen.getByRole("button", { name: "Add script" }));

    expect(onAdd).toHaveBeenCalledWith("/home/user/other.sh");
  });

  it("shows the path the app is about to run before the user confirms", async () => {
    const onAdd = vi.fn(async () => undefined);
    const pickPath = vi.fn(async () => null);

    render(<AddScriptForm onAdd={onAdd} pickPath={pickPath} />);

    await userEvent.type(screen.getByRole("textbox"), "/home/user/run.sh");
    await userEvent.click(screen.getByRole("button", { name: "Add script" }));

    expect(screen.getByText(/about to register/i)).toBeInTheDocument();
    expect(screen.getByText("/home/user/run.sh")).toBeInTheDocument();
    expect(onAdd).toHaveBeenCalledWith("/home/user/run.sh");
  });

  it("does not submit an empty path", async () => {
    const onAdd = vi.fn(async () => undefined);
    const pickPath = vi.fn(async () => null);

    render(<AddScriptForm onAdd={onAdd} pickPath={pickPath} />);

    await userEvent.click(screen.getByRole("button", { name: "Add script" }));

    expect(onAdd).not.toHaveBeenCalled();
  });
});
