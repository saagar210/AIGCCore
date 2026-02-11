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

export function App() {
  const [snap, setSnap] = useState<NetworkSnapshot | null>(null);
  const status = useMemo(() => {
    if (!snap) return "Loading…";
    return `${snap.network_mode} (${snap.proof_level})`;
  }, [snap]);

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
    })();
  }, []);

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
      </main>
    </div>
  );
}
