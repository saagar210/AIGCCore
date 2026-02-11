# Annex A — Evidence Bundle v1 Specification (Locked)

**Status:** Locked v1.0.0  
**Purpose:** A portable, verifiable, versioned export package that proves a Run’s inputs, policies, model snapshots, outputs, and evaluation status.

---

## A.1 Bundle goals
- Portable: single zip file.
- Verifiable: hashes + manifest + tamper-evident logs.
- Versioned: forward/backward compatibility via `bundle_version`.
- Pack-agnostic: identical structure across packs.
- Deterministic where it matters: deliverables avoid volatile timestamps; metadata may include timestamps.

---

## A.2 Naming
- `evidence_bundle_<run_id>_v1.zip`

---

## A.3 Directory layout (inside the zip)
```text
/BUNDLE_INFO.json
/run_manifest.json
/audit_log.ndjson
/eval_report.json
/artifact_hashes.csv
/exports/<pack_id>/
  /deliverables/
  /attachments/
    citations_map.json
    redactions_map.json
    templates_used.json
/inputs_snapshot/
  artifact_list.json
  policy_snapshot.json
  network_snapshot.json
  model_snapshot.json
```

---

## A.4 Required files (and why)
- `BUNDLE_INFO.json` — entrypoint, schema versions, build metadata.
- `run_manifest.json` — canonical record of run inputs/config/outputs.
- `audit_log.ndjson` — tamper-evident event chain for the run.
- `eval_report.json` — test suite results and gate status.
- `artifact_hashes.csv` — chain-of-custody for inputs.
- `exports/**` — deliverables + machine-readable maps.
- `inputs_snapshot/**` — frozen configuration snapshots.

---

## A.5 Determinism rules
- Deliverables SHOULD NOT include “generated at <time>” strings.
- Use `run_id` and content hashes for traceability.
- Timestamps belong in manifests and bundle info.

---

## A.6 Minimum validation rules (bundle is “valid” iff)
- All required files exist.
- All listed input/output hashes verify.
- Audit log hash chain verifies (no breaks).
- Eval report exists and includes `overall_status` and named gates.
- If policy requires citations, `citations_map.json` exists and validates.

---

## A.7 Schema highlights
### `BUNDLE_INFO.json`
- `bundle_version`: "1.0.0"
- `schema_versions`: { run_manifest, eval_report, citations_map, redactions_map }
- `pack_id`, `pack_version`, `core_build`, `run_id`

### `run_manifest.json`
- `inputs[]`: { artifact_id, sha256, bytes, mime_type, logical_role }
- `outputs[]`: { path, sha256, bytes, content_type, logical_role }
- `model_calls[]`: summary (no raw prompts required)
- `eval.gate_status`: PASS|FAIL|WARN

### `audit_log.ndjson`
- per event: `ts_utc`, `event_type`, `run_id`, `vault_id`, `actor`, `details`, `prev_event_hash`, `event_hash`

### `eval_report.json`
- `overall_status`: PASS|FAIL|WARN
- `tests[]`: { test_id, category, status, details }
- `gates[]`: named gate results

### `citations_map.json`
- output_path → list of { artifact_id, locator_type, locator }

### `redactions_map.json`
- artifact_id → list of redaction regions + method + reason
