import type { PackCommandStatus } from "./types";

type Props = {
  running: boolean;
  result: PackCommandStatus | null;
  error: string | null;
  onRun: () => Promise<void>;
};

export function FinanceOSPanel({ running, result, error, onRun }: Props) {
  return (
    <section className="card">
      <h2>Phase 6 FinanceOS (Stage 0 wiring)</h2>
      <p>Retention-aware scaffolding and contract boundaries are wired for execution.</p>
      <button type="button" disabled={running} onClick={() => void onRun()}>
        {running ? "Running FinanceOS..." : "Run FinanceOS Scaffold"}
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
