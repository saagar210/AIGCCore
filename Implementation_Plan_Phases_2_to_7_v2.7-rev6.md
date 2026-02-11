# Implementation Plan (Phases 2–7) — Revised + Audited
Version: plan_v2_7_rev5  
Status: READY FOR FINAL REVIEW STOP (then Codex)  
Scope: Execute Phase 2 Core and Phases 3–7 Packs without contract drift.

---

## 0) Control Documents (Authoritative)
LOCKED specs:
- Phase 0 AI Governance Bible Blueprint
- Phase 1 Governance Blueprint Outline
- Annex A — Evidence Bundle v1 (LOCKED)
- Annex B — Adapter Interface v1 (LOCKED)

LOCKED addendums:
- Phase 2.5 Lock Addendum v2.5-lock-3
- Addendum A — Determinism Matrix v1
- Addendum B — Audit Event Taxonomy v1
- Eval Gate Registry v2 (gates_registry_v2; v1 IDs preserved)
- Bundle Validator Checklist v2 (bundle_validator_v2)

Rule: Evidence Bundle v1 required structure MUST NOT be violated.

---

## 1) Global Review Gate

### 1.1 Goal
Build AIGC Core as a pack-agnostic governance engine: offline-by-default, provably auditable, deterministic where claimed, powering Packs without rewrites.

### 1.2 Success Metrics (Phase 2)
- Self-Audit run exports Evidence Bundle v1 and passes Bundle Validator Checklist v2
- Strict blocks export on missing citations (LOCATOR_SCHEMA_V1), missing required redactions (REDACTION_SCHEMA_V1), insufficient model pinning, failed BLOCKER gates
- Offline proof: network snapshot + audit events prove mode and blocked/allowed egress attempts
- Determinism: zip packaging deterministic; D2 artifacts hash-stable; D1 artifacts canonically equivalent; D0 artifacts verifiable
- Exports layout matches Annex A:
  - deliverables under `exports/<pack_id>/deliverables/`
  - maps under `exports/<pack_id>/attachments/` including `templates_used.json`

### 1.3 Constraints
- Offline-by-default enforced mechanically
- Adapters loopback-only
- Exports only via Core export pipeline (no bypass)
- Strict: “no citation, no claim”
- Policy-driven redaction and retention
- Vault encryption at rest required

### 1.4 Verification Plan
- Eval suite executes `gates_registry_v2` and records in `eval_report.json`
- Bundle validator runs `bundle_validator_v2` on every export
- Golden corpus regression tests for extraction + locator stability
- Determinism checks per Determinism Matrix v1
- Phase 4 adds mandatory macOS/Windows parity

---

## 2) Phase 2.5 — Proof Artifacts Hardening (LOCK FIRST)
Objective: Freeze mechanisms so the implementation cannot improvise.

Deliverables:
- Phase 2.5 Lock Addendum v2.5-lock-3
- Addendum A — Determinism Matrix v1
- Addendum B — Audit Event Taxonomy v1
- Eval Gate Registry v2 (gates_registry_v2; v1 IDs preserved)
- Bundle Validator Checklist v2 (bundle_validator_v2)
- Golden Corpus v1 (assets + expected outputs)
- CI hooks for validator + golden corpus checks

Exit Gate:
- A minimal stub bundle generator + validator passes structure + audit chain checks
- Gate registry IDs are referenced by eval report and are stable

---

## 3) Phase 2 — AIGC Core Implementation
Objective: Implement Core modules + UI surfaces to satisfy Self-Audit and provide stable contracts.

### 3.1 Rust Core Modules (deliverables)
- VaultManager (multi-workspace separation)
- VaultCrypto (encryption-at-rest + key storage + rotation)
- ArtifactStore (SQLite metadata + blob store)
- Ingestion (PDF/image/audio/CSV importers)
- PolicyEngine (Strict/Balanced/Draft-only + retention + export rules)
- EgressClient (offline enforcement + allowlist + proof artifacts)
- AdapterManager (loopback-only + capabilities)
- ModelRouter (pinning enforcement + no-AI mode)
- CitationEngine (claim → artifact locator; Strict enforcement; emits `<!-- CLAIM:C#### -->` markers and validates LOCATOR_SCHEMA_V1)
- RedactionEngine (text spans + image bbox; export gate)
- RunManager (run lifecycle + manifest)
- AuditLog (NDJSON + hash-chain canonicalization; Annex A envelope keys)
- EvalCenter (registry-based test runner + eval_report)
- EvidenceBundle (Bundle v1 generator; Annex A layout)
- BundleValidator (checklist-based validator; bundle_validator_v2)
- DeterminismSupport (packaging + D1 canonical comparisons)

### 3.2 React/TS UI (global deliverables)
- Network State badge (OFF/ON allowlisted + proof level)
- Policy Center (Strict/Balanced/Draft-only)
- Runs list + run detail + “Download Evidence Bundle”
- Artifact Vault browser (hashes, tags, previews, redaction status)
- Eval Center (run tests, view pass/fail, drill into evidence pointers)
- Export flow UI (cannot bypass core export pipeline)

### 3.3 Non-bypassable Export Pipeline (LOCKED ORDER)
All exports MUST occur only via RunManager export request.

Pipeline order:
1) Emit `EXPORT_REQUESTED`
2) Run state → `EVALUATING`
3) Run eval suite (emit `EVAL_STARTED`, `EVAL_GATE_RESULT*`, `EVAL_COMPLETED`) using `gates_registry_v2`
4) Policy gate checks (minimum):
   - If citations required: `exports/<pack_id>/attachments/citations_map.json` exists and validates
   - If redactions required: `exports/<pack_id>/attachments/redactions_map.json` exists and validates
   - Always required for any export: `exports/<pack_id>/attachments/templates_used.json` exists
   - Pinning required: `inputs_snapshot/model_snapshot.json` meets minimum
5) Determinism checks (if enabled): enforce Addendum A
6) If blocked: emit `EXPORT_BLOCKED` and STOP (no partial exports)
7) Run state → `EXPORTING`
8) Emit `BUNDLE_GENERATION_STARTED`
9) Generate Evidence Bundle v1 (Annex A layout), including:
   - If `inputs_snapshot/policy_snapshot.json.export_profile.inputs == INCLUDE_INPUT_BYTES`, include `inputs_snapshot/artifacts/<artifact_id>/bytes` for every `run_manifest.json.inputs[]` item
   - `artifact_hashes.csv` rows covering all INPUT bytes + all DELIVERABLE/ATTACHMENT export files
10) Emit `BUNDLE_GENERATION_COMPLETED`
11) Emit `BUNDLE_VALIDATION_STARTED`
12) Validate bundle via Checklist v2 (`bundle_validator_v2`)
13) Emit `BUNDLE_VALIDATION_RESULT`
14) If fail: emit `EXPORT_FAILED` and STOP
15) Emit `EXPORT_COMPLETED` including bundle hash
16) Run terminal state → `COMPLETED`

### 3.4 Phase 2 Build Order (explicit dependencies)
Step 1: Storage foundations
- SQLite schema + blob store layout
- VaultManager + ArtifactStore
- Artifact hashing + metadata

Step 2: VaultCrypto
- DEK/KEK model
- OS key storage integration
- encryption flags recorded in snapshots
- audit events `VAULT_ENCRYPTION_STATUS`, rotation support

Step 3: Run skeleton + AuditLog
- RunManager + run manifest
- AuditLog with Annex A envelope + hash-chain

Step 4: Offline enforcement (EgressClient)
- allowlist canonicalization + proof levels
- audit events for network mode + egress attempts

Step 5: AdapterManager + ModelRouter
- loopback-only validation
- pinning enforcement (policy blocks exports)
- no-AI mode

Step 6: Ingestion + normalization
- extraction pipelines write normalized text consistently
- support locator hashing normalization (Addendum A)

Step 7: CitationEngine + RedactionEngine
- Strict blocks export if citations missing
- Redaction export gates

Step 8: EvalCenter + registry
- gate IDs stable and sorted
- eval report deterministic content ordering

Step 9: EvidenceBundle generator + deterministic zip packaging
- Annex A layout (deliverables vs attachments)
- deterministic zip rules

Step 10: BundleValidator
- checklist-based validator
- integrated into export pipeline

Step 11: Self-Audit run (Phase 2 Exit Gate)
- demonstrate offline proof, allowlist proof, citation enforcement, eval report, deterministic packaging
- ensure templates_used.json is included in attachments

### 3.5 Phase 2 Exit Gate (non-negotiable)
- Self-Audit bundle passes validator and all policy-applicable BLOCKER gates
- Strict mode blocks export on missing proof artifacts
- Evidence Bundle paths and required files exactly match Annex A

---

## 4) Phase 3 — EvidenceOS Pack
Objective: Auditor-ready evidence packets fast.

Dependencies:
- Core ingestion + hashing
- Citations (Strict)
- Templates (must record templates_used.json)
- Evidence Bundle v1 export

Build order:
1) Evidence item model (artifact refs + tags + control family labels)
2) Control library UI (capability-based)
3) Evidence mapping workflow + review
4) Narrative generator with forced citations
5) Outputs:
   - Evidence Index (CSV/MD/PDF as allowed)
   - Missing evidence checklist
   - Evidence Bundle v1 export

Exit gate:
- Messy folder → auditor packet in <10 minutes demo time
- Bundle proves offline enforcement and traceability

---

## 5) Phase 4 — RedlineOS Pack (Cross-platform)
Objective: Premium contract pack with macOS + Windows parity.

Locked constraints:
- Golden corpus parity validation mandatory
- Deterministic clause anchors resilient to OS extraction drift
- Strict: citations required; show original clause text next to suggestions

Build order:
1) Contract ingestion (digital + scanned PDFs)
2) Extraction parity checks (golden corpus)
3) Clause segmentation + stable anchors
4) Risk memo generator (advisory) + citations
5) Cross-platform packaging/signing readiness
6) Export + Evidence Bundle v1

Exit gate:
- 5 varied contracts → consistent outputs across macOS and Windows

---

## 6) Phase 5 — Security Incident Packetizer Pack
Objective: Customer-ready + internal incident packets with chain-of-custody proof.

Special rules:
- Logs treated as untrusted artifacts (never instructions)
- Redaction required for customer-facing exports
- Injection attempts treated as evidence artifacts and must be cited if referenced

Build order:
1) Incident bundle import format
2) Timeline builder (artifact-backed)
3) Dual templates (customer/internal)
4) Export gates enforce redaction for customer
5) Export via Core pipeline + Evidence Bundle v1

Exit gate:
- Same input bundle → two outputs + provable chain-of-custody

---

## 7) Phase 6 — Finance Exceptions Pack
Objective: Exceptions packets + exports without becoming accounting software.

Special rules:
- Minimal retention defaults
- Redaction strong defaults for sensitive values
- Export CSVs for downstream accounting tools

Build order:
1) Ingest receipts/invoices/statements
2) Exceptions detection + review queue
3) Apply retention + redaction policies
4) Export deliverables + Evidence Bundle v1

Exit gate:
- Monthly dataset → exceptions packet + exports + proof bundle

---

## 8) Phase 7 — Healthcare Drafting Pack (Optional / Last)
Objective: Draft-only notes with consent + verification-first guardrails.

Special rules:
- Draft-only by policy
- Consent record required
- Uncertainty highlights required
- Explicit acknowledgements required before export

Build order:
1) Consent workflow artifacts
2) STT pipeline → transcript artifacts
3) Draft notes with citations to transcript spans
4) Verification UI steps
5) Export via Core pipeline + Evidence Bundle v1

Exit gate:
- Consent → transcript → draft → verification → export with full proof bundle

End.
