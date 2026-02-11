# Codex One-Shot Driver Prompt v2

You are implementing **Phase 2: AIGC Core** for a local-first, offline-by-default, provably auditable desktop app.

## 0) Authority & Contract (do not drift)
**Authoritative LOCKED specs (must comply):**
- `Phase_0_AI_Governance_Bible_Blueprint.md`
- `Phase_1_Governance_Blueprint_Outline.md`
- `Annex_A_Evidence_Bundle_v1_Spec.md` (Evidence Bundle v1 is LOCKED)
- `Annex_B_Adapter_Interface_v1_Spec.md` (Adapter Interface v1 is LOCKED)

**Working locks for implementation (must comply):**
- `Phase_2_5_Lock_Addendum_v2.5-lock-4.md` (LOCKED)
- `Addendum_A_Determinism_Matrix_v1.md` (LOCKED)
- `Addendum_B_Audit_Event_Taxonomy_v1.md` (LOCKED)
- `Eval_Gate_Registry_v3.md` (LOCKED; v1 gate IDs preserved)
- `Bundle_Validator_Checklist_v3.md` (LOCKED)

**Implementation plan (execute in order):**
- `Implementation_Plan_Phases_2_to_7_v2.7-rev6.md`

If any requirement is unclear or contradictory **in these files**, create a single `STOP_REPORT.md` describing:
- exact file + section
- why it is contradictory
- the minimal resolution needed
Then STOP.

---

## REVIEW GATE (before coding)
### Goal
Implement Phase 2 Core so a **Self-Audit run** produces an **Evidence Bundle v1** that passes:
- `bundle_validator_v2` checklist
- all policy-applicable **BLOCKER** gates from `gates_registry_v2`

### Success metrics
- Offline-by-default is mechanically enforced (Core egress boundary + UI remote fetch disabled + loopback-only adapters) and proven via snapshots + audit events.
- Audit log is NDJSON + hash chained with canonicalization ID `PHASE_2_5_LOCK_ADDENDUM_V2_5_LOCK_3`.
- Evidence Bundle v1 export includes required structure + required attachments paths.
- Determinism (when enabled) matches Addendum A; deterministic zip packaging rules enforced.
- Strict mode blocks export on:
  - missing claim markers / missing citations (LOCATOR_SCHEMA_V1)
  - missing required redactions (REDACTION_SCHEMA_V1)
  - insufficient model pinning
  - failed BLOCKER gates

### Constraints
- No network egress except through the single `EgressClient` + allowlist rules.
- Adapters must be `127.0.0.1` only; reject anything else.
- Exports only via the Core export pipeline (no bypass).
- Evidence Bundle v1 structure is not negotiable; additions must be additive-only (e.g., `artifacts/inputs/...`).

### Must / Should / Could
**MUST**
- Implement core modules needed for Phase 2 exit gate (RunManager, AuditLog, PolicyEngine, EgressClient, AdapterManager, ModelRouter, CitationEngine, RedactionEngine, EvalCenter, EvidenceBundle, BundleValidator, VaultCrypto, ArtifactStore).
- Implement schemas: LOCATOR_SCHEMA_V1, REDACTION_SCHEMA_V1, TEMPLATES_USED_V1 per Lock Addendum.
- Implement deterministic zip packaging rules (sorted paths, mtime=0, DEFLATE level 9).

**SHOULD**
- Provide a CLI/dev command to run Self-Audit without UI interaction for CI.
- Include unit tests for canonicalization, hashing, and schema validation.

**COULD**
- Add minimal UI surfaces (Network badge, Runs list, Export button) if already scaffolded; otherwise keep UI thin and focus on Core correctness.

### Stop / Go
**GO** if you can implement all MUST items with the current packet.
**STOP** only if a real contradiction exists in the packet.

### Verification plan
- Add a `self_audit` fixture that:
  - creates a vault
  - ingests a small sample artifact set (at least 1 PDF or text file)
  - runs in STRICT policy
  - generates a simple markdown deliverable with claim markers + citations
  - exports Evidence Bundle v1
  - runs `bundle_validator_v2`
  - runs eval suite using `gates_registry_v2`
  - asserts export is BLOCKED on any BLOCKER failure

Proceed.

---

## 1) Implementation phases (do in this order)

### Phase A — Project scaffolding + module boundaries
Create/confirm a Rust core layout with explicit module boundaries matching Phase 1:
- `vault/`, `storage/`, `crypto/`, `audit/`, `policy/`, `egress/`, `adapters/`, `models/`, `citations/`, `redactions/`, `eval/`, `bundle/`, `determinism/`, `runs/`

Add:
- strict linting and formatting
- centralized error type and error categories used by audit events

### Phase B — Offline enforcement (mechanical)
Implement the `EgressClient` and enforce:
- no HTTP client creation outside `egress/`
- UI cannot load remote pages (Tauri config + CSP)
- all UI fetches go through `tauri::invoke` -> Rust -> `EgressClient`

Add a CI/lint gate:
- `clippy.toml` + `#![deny(clippy::disallowed_types)]` to disallow `reqwest::Client`, `ureq::*`, `std::net::*` outside `egress/`

Emit audit events per Addendum B:
- `NETWORK_MODE_SET`
- `ALLOWLIST_UPDATED`
- `EGRESS_REQUEST_ALLOWED` / `EGRESS_REQUEST_BLOCKED`

### Phase C — Audit log + hash chain
Implement `audit_log.ndjson`:
- required envelope keys only (additional keys only in `details.meta`)
- canonicalization ID `PHASE_2_5_LOCK_ADDENDUM_V2_5_LOCK_3`
- SHA-256 event hash chain

### Phase D — Storage + crypto
Implement:
- SQLite metadata + blob store
- per-vault encryption at rest (XChaCha20-Poly1305 preferred; AES-256-GCM allowed)
- key storage via OS primitives (macOS Keychain / Windows DPAPI)
Emit:
- `VAULT_ENCRYPTION_STATUS`
- `VAULT_KEY_ROTATED`

### Phase E — Runs + manifests + snapshots
Implement:
- `run_id` derivation rules (Lock Addendum §3.2)
- `run_manifest.json` with inputs/outputs + determinism fields (Addendum A)
- `inputs_snapshot/*.json` minimum schemas (Lock Addendum §4.4)
- `BUNDLE_INFO.json` minimum schema (Lock Addendum §4.5)

### Phase F — Citations + redactions
Implement:
- claim marker parsing for `<!-- CLAIM:C#### -->`
- `citations_map.json` (LOCATOR_SCHEMA_V1)
- `redactions_map.json` (REDACTION_SCHEMA_V1)
- redaction gating rule: restricted/tagged artifacts referenced by citations must be covered (Lock Addendum §6.3)
Emit:
- `CITATION_VALIDATION_RESULT`
- `REDACTION_VALIDATION_RESULT`

### Phase G — Eval suite + registry
Implement:
- `gates_registry_v2` loader
- eval runner emitting:
  - `EVAL_STARTED`
  - `EVAL_GATE_RESULT`
  - `EVAL_COMPLETED`
- `eval_report.json` with deterministic ordering (sort by gate_id)

### Phase H — Evidence Bundle v1 generator + validator
Implement:
- export directory layout per Annex A
- include raw input bytes under `artifacts/inputs/<artifact_id>/bytes`
- generate `artifact_hashes.csv` per Lock Addendum §4.2
- deterministic zip packaging rules (Lock Addendum + Addendum A)
- run `bundle_validator_v2` on every export attempt
Emit:
- `BUNDLE_GENERATION_STARTED/COMPLETED`
- `BUNDLE_VALIDATION_STARTED/RESULT`
- `EXPORT_COMPLETED` or `EXPORT_BLOCKED` / `EXPORT_FAILED`

---

## 2) Deliverables (what you must produce)
- Working Phase 2 Core implementation
- A `self_audit` command or test that produces an Evidence Bundle v1 zip and validates it
- Unit tests for:
  - audit hash-chain canonicalization
  - allowlist canonicalization + hashing
  - run_id derivation
  - schema validators for LOCATOR_SCHEMA_V1 / REDACTION_SCHEMA_V1 / TEMPLATES_USED_V1
  - deterministic zip packaging (mtime=0, lex sort, DEFLATE level 9)

Stop only if truly necessary per the STOP rules above.
