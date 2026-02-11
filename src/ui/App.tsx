import React, { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FinanceOSPanel } from "./packs/FinanceOSPanel";
import { HealthcareOSPanel } from "./packs/HealthcareOSPanel";
import { IncidentOSPanel } from "./packs/IncidentOSPanel";
import { RedlineOSPanel } from "./packs/RedlineOSPanel";
import type { PackCommandStatus } from "./packs/types";

type NetworkSnapshot = {
  network_mode: "OFFLINE" | "ONLINE_ALLOWLISTED";
  proof_level:
    | "OFFLINE_STRICT"
    | "ONLINE_ALLOWLIST_CORE_ONLY"
    | "ONLINE_ALLOWLIST_WITH_OS_FIREWALL_PROFILE";
  ui_remote_fetch_disabled: boolean;
};

type ControlDefinition = {
  control_id: string;
  title: string;
  capability: string;
  control_family: string;
  description: string;
};

type EvidenceOsRunResult = {
  status: string;
  bundle_path: string;
  bundle_sha256: string;
  missing_control_ids: string[];
};

type EvidenceOsRunInput = {
  enabled_capabilities: string[];
  artifact_title: string;
  artifact_body: string;
  artifact_tags_csv: string;
  control_families_csv: string;
  claim_text: string;
};

export function App() {
  const [snap, setSnap] = useState<NetworkSnapshot | null>(null);
  const [controls, setControls] = useState<ControlDefinition[]>([]);
  const [runResult, setRunResult] = useState<EvidenceOsRunResult | null>(null);
  const [runError, setRunError] = useState<string | null>(null);
  const [running, setRunning] = useState(false);
  const [selectedCapability, setSelectedCapability] = useState("ALL");
  const [artifactTitle, setArtifactTitle] = useState("Network policy evidence");
  const [artifactBody, setArtifactBody] = useState(
    "Audit log excerpt proving offline mode and blocked egress."
  );
  const [artifactTags, setArtifactTags] = useState("OPS,NETWORK");
  const [controlFamilies, setControlFamilies] = useState(
    "Auditability,NetworkGovernance,Traceability"
  );
  const [claimText, setClaimText] = useState(
    "The run stayed offline and blocked non-allowlisted egress requests."
  );

  const [futurePackRunning, setFuturePackRunning] = useState<string | null>(null);
  const [futurePackResult, setFuturePackResult] = useState<Record<string, PackCommandStatus>>({});
  const [futurePackError, setFuturePackError] = useState<Record<string, string>>({});

  const status = useMemo(() => {
    if (!snap) return "Loading…";
    return `${snap.network_mode} (${snap.proof_level})`;
  }, [snap]);

  const capabilities = useMemo(() => {
    const all = controls.map((control) => control.capability);
    return ["ALL", ...Array.from(new Set(all)).sort()];
  }, [controls]);

  useEffect(() => {
    (async () => {
      try {
        const s = await invoke<NetworkSnapshot>("get_network_snapshot");
        setSnap(s);
      } catch {
        setSnap({
          network_mode: "OFFLINE",
          proof_level: "OFFLINE_STRICT",
          ui_remote_fetch_disabled: true
        });
      }

      try {
        const list = await invoke<ControlDefinition[]>("list_control_library");
        setControls(list);
      } catch (error) {
        setRunError(`Failed to load control library: ${String(error)}`);
      }
    })();
  }, []);

  const filteredControls = useMemo(() => {
    if (selectedCapability === "ALL") return controls;
    return controls.filter((control) => control.capability === selectedCapability);
  }, [controls, selectedCapability]);

  const onRunEvidenceOs = async () => {
    setRunning(true);
    setRunError(null);
    try {
      const payload: EvidenceOsRunInput = {
        enabled_capabilities:
          selectedCapability === "ALL" ? [] : [selectedCapability],
        artifact_title: artifactTitle,
        artifact_body: artifactBody,
        artifact_tags_csv: artifactTags,
        control_families_csv: controlFamilies,
        claim_text: claimText
      };
      const result = await invoke<EvidenceOsRunResult>(
        "generate_evidenceos_bundle",
        { input: payload }
      );
      setRunResult(result);
    } catch (error) {
      setRunError(String(error));
      setRunResult(null);
    } finally {
      setRunning(false);
    }
  };

  const runFuturePack = async (command: string, input: unknown) => {
    setFuturePackRunning(command);
    setFuturePackError((prev) => ({ ...prev, [command]: "" }));
    try {
      const result = await invoke<PackCommandStatus>(command, { input });
      setFuturePackResult((prev) => ({ ...prev, [command]: result }));
    } catch (error) {
      setFuturePackError((prev) => ({ ...prev, [command]: String(error) }));
      setFuturePackResult((prev) => ({ ...prev, [command]: { status: "FAILED", message: String(error) } }));
    } finally {
      setFuturePackRunning(null);
    }
  };

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">AIGC Core</div>
        <div className="badge" data-mode={snap?.network_mode ?? "UNKNOWN"}>
          Network: <strong>{status}</strong>
        </div>
      </header>

      <main className="main">
        <section className="card">
          <h2>Phase 2 Hard Guarantees</h2>
          <ul>
            <li>Offline-by-default enforced in Rust core</li>
            <li>Hash-chained canonical audit log</li>
            <li>Deterministic Evidence Bundle v1 export</li>
            <li>Validator checklist + eval gates runnable locally</li>
          </ul>
        </section>

        <section className="card">
          <h2>Phase 3 EvidenceOS Pack</h2>
          <p>
            Capability-based control mapping and strict-citation narrative export through the Core export pipeline.
          </p>
          <div className="row">
            <label htmlFor="capability-filter">Capability</label>
            <select
              id="capability-filter"
              value={selectedCapability}
              onChange={(event) => setSelectedCapability(event.target.value)}
            >
              {capabilities.map((capability) => (
                <option key={capability} value={capability}>
                  {capability}
                </option>
              ))}
            </select>
          </div>
          <div className="form-grid">
            <label htmlFor="artifact-title">Artifact title</label>
            <input
              id="artifact-title"
              value={artifactTitle}
              onChange={(event) => setArtifactTitle(event.target.value)}
            />
            <label htmlFor="artifact-body">Artifact text</label>
            <textarea
              id="artifact-body"
              rows={3}
              value={artifactBody}
              onChange={(event) => setArtifactBody(event.target.value)}
            />
            <label htmlFor="artifact-tags">Artifact tags (CSV)</label>
            <input
              id="artifact-tags"
              value={artifactTags}
              onChange={(event) => setArtifactTags(event.target.value)}
            />
            <label htmlFor="control-families">Control families (CSV)</label>
            <input
              id="control-families"
              value={controlFamilies}
              onChange={(event) => setControlFamilies(event.target.value)}
            />
            <label htmlFor="claim-text">Narrative claim</label>
            <textarea
              id="claim-text"
              rows={3}
              value={claimText}
              onChange={(event) => setClaimText(event.target.value)}
            />
          </div>
          <div className="controls-grid">
            {filteredControls.map((control) => (
              <article key={control.control_id} className="control-card">
                <h3>{control.control_id}</h3>
                <p className="control-title">{control.title}</p>
                <p className="meta">
                  {control.capability} / {control.control_family}
                </p>
                <p>{control.description}</p>
              </article>
            ))}
          </div>
          <button type="button" disabled={running} onClick={onRunEvidenceOs}>
            {running ? "Generating EvidenceOS Bundle…" : "Generate EvidenceOS Bundle"}
          </button>
          {runError && <p className="error">Phase 3 run failed: {runError}</p>}
          {runResult && (
            <div className="result">
              <p>
                Export status: <strong>{runResult.status}</strong>
              </p>
              <p>Bundle path: {runResult.bundle_path}</p>
              <p>Bundle SHA-256: {runResult.bundle_sha256}</p>
              <p>
                Missing controls:{" "}
                {runResult.missing_control_ids.length > 0
                  ? runResult.missing_control_ids.join(", ")
                  : "None"}
              </p>
            </div>
          )}
        </section>

        <RedlineOSPanel
          running={futurePackRunning === "run_redlineos"}
          result={futurePackResult.run_redlineos ?? null}
          error={futurePackError.run_redlineos ?? null}
          onRun={() =>
            runFuturePack("run_redlineos", {
              schema_version: "REDLINEOS_INPUT_V1",
              contract_artifacts: [{ artifact_id: "a_demo", sha256: "demo", filename: "contract.pdf" }],
              extraction_mode: "OCR",
              jurisdiction_hint: "US-CA",
              review_profile: "default"
            })
          }
        />
        <IncidentOSPanel
          running={futurePackRunning === "run_incidentos"}
          result={futurePackResult.run_incidentos ?? null}
          error={futurePackError.run_incidentos ?? null}
          onRun={() =>
            runFuturePack("run_incidentos", {
              schema_version: "INCIDENTOS_INPUT_V1",
              incident_artifacts: [{ artifact_id: "i_demo", sha256: "demo", source_type: "syslog" }],
              timeline_start_hint: null,
              timeline_end_hint: null,
              customer_redaction_profile: "strict"
            })
          }
        />
        <FinanceOSPanel
          running={futurePackRunning === "run_financeos"}
          result={futurePackResult.run_financeos ?? null}
          error={futurePackError.run_financeos ?? null}
          onRun={() =>
            runFuturePack("run_financeos", {
              schema_version: "FINANCEOS_INPUT_V1",
              finance_artifacts: [{ artifact_id: "f_demo", sha256: "demo", artifact_kind: "invoice" }],
              period: "2026-01",
              exception_rules_profile: "default",
              retention_profile: "ret_min"
            })
          }
        />
        <HealthcareOSPanel
          running={futurePackRunning === "run_healthcareos"}
          result={futurePackResult.run_healthcareos ?? null}
          error={futurePackError.run_healthcareos ?? null}
          onRun={() =>
            runFuturePack("run_healthcareos", {
              schema_version: "HEALTHCAREOS_INPUT_V1",
              consent_artifacts: [{ artifact_id: "c_demo", sha256: "demo", artifact_kind: "consent" }],
              transcript_artifacts: [{ artifact_id: "t_demo", sha256: "demo", artifact_kind: "transcript" }],
              draft_template_profile: "soap",
              verifier_identity: "clinician_1"
            })
          }
        />
      </main>
    </div>
  );
}
