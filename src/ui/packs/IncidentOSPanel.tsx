import type { PackCommandStatus } from "./types";

type Props = {
  running: boolean;
  result: PackCommandStatus | null;
  error: string | null;
  onRun: () => Promise<void>;
};

export function IncidentOSPanel({ running, result, error, onRun }: Props) {
  return (
    <section className="card">
      <h2>Phase 5 IncidentOS (Stage 0 wiring)</h2>
      <p>Untrusted-log path and command scaffolding are now wired for implementation start.</p>
      <button type="button" disabled={running} onClick={() => void onRun()}>
        {running ? "Running IncidentOS..." : "Run IncidentOS Scaffold"}
      </button>
      {error && <p className="error">{error}</p>}
      {result && (
        <div className="result">
          <p>Status: {result.status}</p>
          <p>{result.message}</p>
        </div>
      )}
    </section>
  );
}
