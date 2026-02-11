import React, { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

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

export function App() {
  const [snap, setSnap] = useState<NetworkSnapshot | null>(null);
  const [controls, setControls] = useState<ControlDefinition[]>([]);
  const [runResult, setRunResult] = useState<EvidenceOsRunResult | null>(null);
  const [runError, setRunError] = useState<string | null>(null);
  const [running, setRunning] = useState(false);
  const [selectedCapability, setSelectedCapability] = useState("ALL");
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
      const result = await invoke<EvidenceOsRunResult>("generate_evidenceos_bundle_demo");
      setRunResult(result);
    } catch (error) {
      setRunError(String(error));
      setRunResult(null);
    } finally {
      setRunning(false);
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
            {running ? "Generating EvidenceOS Bundle…" : "Generate EvidenceOS Demo Bundle"}
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
      </main>
    </div>
  );
}
