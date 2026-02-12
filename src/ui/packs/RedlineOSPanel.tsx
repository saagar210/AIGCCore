import React, { useState } from "react";
import type { PackCommandStatus } from "./types";

type Props = {
  running: boolean;
  result: PackCommandStatus | null;
  error: string | null;
  onRun: (input: RedlineOSInput) => Promise<void>;
};

type RedlineOSInput = {
  schema_version: string;
  contract_artifacts: Array<{ artifact_id: string; sha256: string; filename: string }>;
  extraction_mode: "NATIVE_PDF" | "OCR";
  jurisdiction_hint: string | null;
  review_profile: "default" | "aggressive" | "conservative";
};

export function RedlineOSPanel({ running, result, error, onRun }: Props) {
  const [extractionMode, setExtractionMode] = useState<"NATIVE_PDF" | "OCR">("NATIVE_PDF");
  const [jurisdiction, setJurisdiction] = useState<string>("US-CA");
  const [reviewProfile, setReviewProfile] = useState<"default" | "aggressive" | "conservative">("default");

  const handleRun = async () => {
    const input: RedlineOSInput = {
      schema_version: "REDLINEOS_INPUT_V1",
      contract_artifacts: [
        {
          artifact_id: "a_demo_contract",
          sha256: "demo_sha256",
          filename: "contract.pdf",
        },
      ],
      extraction_mode: extractionMode,
      jurisdiction_hint: jurisdiction || null,
      review_profile: reviewProfile,
    };
    await onRun(input);
  };

  return (
    <section className="card">
      <h2>Phase 4: RedlineOS (Contract Review)</h2>
      <p>Extract clauses, assess risks, generate risk memo with citations.</p>

      <div className="form-grid">
        <label htmlFor="extraction-mode">Extraction Mode</label>
        <select
          id="extraction-mode"
          value={extractionMode}
          onChange={(e) => setExtractionMode(e.target.value as "NATIVE_PDF" | "OCR")}
        >
          <option value="NATIVE_PDF">Native PDF (digital)</option>
          <option value="OCR">OCR (scanned)</option>
        </select>

        <label htmlFor="jurisdiction">Jurisdiction Hint</label>
        <input
          id="jurisdiction"
          type="text"
          value={jurisdiction}
          onChange={(e) => setJurisdiction(e.target.value)}
          placeholder="US-CA"
        />

        <label htmlFor="review-profile">Review Profile</label>
        <select
          id="review-profile"
          value={reviewProfile}
          onChange={(e) => setReviewProfile(e.target.value as "default" | "aggressive" | "conservative")}
        >
          <option value="default">Default</option>
          <option value="aggressive">Aggressive</option>
          <option value="conservative">Conservative</option>
        </select>
      </div>

      <button type="button" disabled={running} onClick={handleRun}>
        {running ? "Analyzing Contract..." : "Generate Risk Assessment"}
      </button>

      {error && <p className="error">{error}</p>}
      {result && (
        <div className="result">
          <p>
            Status: <strong>{result.status}</strong>
          </p>
          <p>{result.message}</p>
        </div>
      )}
    </section>
  );
}
