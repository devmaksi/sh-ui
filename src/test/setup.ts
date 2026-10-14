import { cleanup } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";
import { afterEach } from "vitest";

// Vitest runs with globals disabled, so React Testing Library cannot register its own
// automatic cleanup. Without this, the DOM accumulates between tests and queries match
// elements from earlier tests.
afterEach(() => {
  cleanup();
});
