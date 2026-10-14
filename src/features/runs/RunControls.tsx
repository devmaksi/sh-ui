// story: e02s02
// scenario: SC-e02s02-P2-04
interface RunControlsProps {
  running: boolean;
  onStop: () => void;
}

/** Stop is offered only while the run is live; one press ends the whole process group. */
export function RunControls({ running, onStop }: RunControlsProps) {
  return (
    <button
      type="button"
      onClick={onStop}
      disabled={!running}
      className="border border-neutral-900 bg-neutral-900 px-3 py-1 text-sm text-white disabled:opacity-40"
    >
      Stop
    </button>
  );
}
