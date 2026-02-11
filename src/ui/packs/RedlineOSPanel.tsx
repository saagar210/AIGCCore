import type { PackCommandStatus } from "./types";

type Props = {
  running: boolean;
  result: PackCommandStatus | null;
  error: string | null;
  onRun: () => Promise<void>;
};

export function RedlineOSPanel({ running, result, error, onRun }: Props) {
  return (
    <section className="card">
      <h2>Phase 4 RedlineOS (Stage 0 wiring)</h2>
      <p>Command scaffolding is wired through Tauri and Core module boundaries.</p>
      <button type="button" disabled={running} onClick={() => void onRun()}>
        {running ? "Running RedlineOS..." : "Run RedlineOS Scaffold"}
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
