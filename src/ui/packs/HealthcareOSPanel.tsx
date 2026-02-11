import type { PackCommandStatus } from "./types";

type Props = {
  running: boolean;
  result: PackCommandStatus | null;
  error: string | null;
  onRun: () => Promise<void>;
};

export function HealthcareOSPanel({ running, result, error, onRun }: Props) {
  return (
    <section className="card">
      <h2>Phase 7 HealthcareOS (Optional Stage 0 wiring)</h2>
      <p>Consent-gated scaffolding is wired and ready for compliance-gated implementation.</p>
      <button type="button" disabled={running} onClick={() => void onRun()}>
        {running ? "Running HealthcareOS..." : "Run HealthcareOS Scaffold"}
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
