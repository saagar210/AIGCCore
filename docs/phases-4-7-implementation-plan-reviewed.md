# Phases 4–7 Implementation Plan (Reviewed & Execution-Ready)

Date: 2026-02-11  
Reviewer: Senior Software Engineer (30y) + VP Engineering perspective  
Status: **APPROVED WITH REVISIONS**

---

## 1) Executive Review Outcome

This reviewed plan replaces ambiguity in the prior roadmap with explicit architecture, module boundaries, contracts, dependency order, and test gates for Phases 4–7.

### 1.1 Key Gaps Found in Prior Plan

1. Missing concrete module/file boundaries per pack.
2. Missing explicit API contracts between Tauri shell ↔ Core.
3. Missing state machine details for pack workflows.
4. Missing pack-specific eval gate definitions and pass/fail semantics.
5. Missing phased migration strategy for `run_manager` orchestration.
6. Missing edge-case/error handling requirements (especially malformed input, redaction conflicts, consent failure).
7. Missing explicit assumptions and non-goals to prevent scope creep.

### 1.2 What Was Fixed in This Revision

- Added canonical file structure and ownership boundaries.
- Added data models and command contracts with versioning strategy.
- Added stepwise implementation order with hard dependencies and stop/go criteria.
- Added error taxonomy and edge-case handling matrix by phase.
- Added integration test matrix and deterministic validation requirements.
- Added explicit assumptions and judgment calls.

---

## 2) Non-Negotiable Constraints (Inherited)

All future phases **must** preserve:

- Offline-by-default and allowlist enforcement.
- Loopback-only adapters (`127.0.0.1`) with Annex B contract compliance.
- Evidence Bundle v1 layout and required artifacts.
- Determinism mode / ZIP hardening constraints.
- Canonicalized hash-chained audit trail.
- Non-bypassable export pipeline and gate evaluation.

No phase may ship if these regress.

---

## 3) Target Architecture Decisions and Rationale

### 3.1 Architecture Pattern

**Decision:** Keep a layered architecture:

1. **Domain layer (Rust core)**: deterministic business logic, policies, validators, bundle generation.
2. **Application layer (Rust core)**: run orchestration, phase-specific workflow execution.
3. **Transport layer (Tauri commands)**: typed command handlers only; no policy logic here.
4. **Presentation layer (React)**: form/workflow UX only; no direct network egress.

**Rationale:** Preserves auditability, testability, and policy correctness under offline constraints.

### 3.2 Pack Isolation Strategy

**Decision:** Create one Rust module namespace per pack (`redlineos`, `incidentos`, `financeos`, `healthcareos`).

**Rationale:** Maintains pack-agnostic Core while enabling pack-specific workflows and gate additions without coupling.

### 3.3 Contract Versioning

**Decision:** New pack payload schemas adopt `*_V1`; breaking changes require `*_V2` with compatibility adapters.

**Rationale:** Keeps evidence artifacts verifiable over time and avoids silent contract drift.

### 3.4 State Management

**Decision:** Extend `RunState` with pack workflow substates represented in per-pack workflow structs (not global enum explosion).

**Rationale:** Prevents overloading global lifecycle while preserving phase-specific checkpoints.

---

## 4) Exact File Structure and Module Boundaries

### 4.1 Core (`core/src`)

Add:

- `core/src/redlineos/mod.rs`
- `core/src/redlineos/model.rs`
- `core/src/redlineos/workflow.rs`
- `core/src/redlineos/render.rs`
- `core/src/redlineos/anchors.rs`
- `core/src/incidentos/mod.rs`
- `core/src/incidentos/model.rs`
- `core/src/incidentos/workflow.rs`
- `core/src/incidentos/render.rs`
- `core/src/incidentos/sanitize.rs`
- `core/src/financeos/mod.rs`
- `core/src/financeos/model.rs`
- `core/src/financeos/workflow.rs`
- `core/src/financeos/render.rs`
- `core/src/financeos/policies.rs`
- `core/src/healthcareos/mod.rs`
- `core/src/healthcareos/model.rs`
- `core/src/healthcareos/workflow.rs`
- `core/src/healthcareos/render.rs`
- `core/src/healthcareos/consent.rs`

Modify:

- `core/src/lib.rs` (module exports)
- `core/src/run/manager.rs` (pack-aware orchestrated export flow)
- `core/src/eval/registry_v3.json` (new gate IDs)
- `core/src/eval/runner.rs` (gate execution wiring if needed)
- `core/src/validator/mod.rs` (pack-specific validator checks)

### 4.2 Tauri (`src-tauri/src`)

Modify:

- `src-tauri/src/main.rs`
  - Add command handlers:
    - `run_redlineos`
    - `run_incidentos`
    - `run_financeos`
    - `run_healthcareos`
  - Keep handlers as pure DTO mapping + invocation.

### 4.3 UI (`src/ui`)

Add:

- `src/ui/packs/RedlineOSPanel.tsx`
- `src/ui/packs/IncidentOSPanel.tsx`
- `src/ui/packs/FinanceOSPanel.tsx`
- `src/ui/packs/HealthcareOSPanel.tsx`
- `src/ui/packs/types.ts`

Modify:

- `src/ui/App.tsx` (pack routing and run trigger wiring)
- `src/smoke.test.ts` (pack flow smoke tests)

### 4.4 Tools

Modify:

- `tools/gate_runner/src/main.rs` (phase-specific self-audit scenarios)
- `tools/gates/run-all.mjs` (invoke new scenarios)
- `tools/bundle_validator/src/main.rs` (new structural checks as needed)

### 4.5 Docs

Add/modify:

- `docs/spec-compliance-map.md` (Phase 4–7 MUST/SHALL rows)
- `docs/phase-4-closure-report.md`
- `docs/phase-5-closure-report.md`
- `docs/phase-6-closure-report.md`
- `docs/phase-7-closure-report.md` (only if Phase 7 executed)

---

## 5) Data Models, API Contracts, and State Approach

### 5.1 Shared Run Envelope (all packs)

All pack run commands must include:

- `run_id`
- `vault_id`
- `policy_mode`
- `network_mode`
- `proof_level`
- `pinning_level`
- `pack_inputs` (pack-specific schema)

### 5.2 Pack Input/Output Contracts

#### Phase 4: RedlineOS

**Input schema (`REDLINEOS_INPUT_V1`):**
- contract artifacts[]
- extraction_mode
- jurisdiction hint (optional)
- review profile

**Outputs:**
- `exports/redlineos/deliverables/risk_memo.md`
- `exports/redlineos/deliverables/clause_map.csv`
- `exports/redlineos/deliverables/redline_suggestions.md`
- `exports/redlineos/attachments/citations_map.json`
- `exports/redlineos/attachments/anchor_index.json`

#### Phase 5: IncidentOS

**Input schema (`INCIDENTOS_INPUT_V1`):**
- incident artifacts[]
- source type metadata
- timeline start/end hints (optional)
- customer redaction policy profile

**Outputs:**
- `exports/incidentos/deliverables/customer_packet.md`
- `exports/incidentos/deliverables/internal_packet.md`
- `exports/incidentos/deliverables/timeline.csv`
- `exports/incidentos/attachments/redactions_map.json`
- `exports/incidentos/attachments/citations_map.json`

#### Phase 6: FinanceOS

**Input schema (`FINANCEOS_INPUT_V1`):**
- statement/invoice/receipt artifacts[]
- period metadata
- exception rules profile
- retention profile

**Outputs:**
- `exports/financeos/deliverables/exceptions_packet.md`
- `exports/financeos/deliverables/exceptions.csv`
- `exports/financeos/deliverables/accounting_export.csv`
- `exports/financeos/attachments/redactions_map.json`
- `exports/financeos/attachments/citations_map.json`

#### Phase 7: HealthcareOS (optional)

**Input schema (`HEALTHCAREOS_INPUT_V1`):**
- consent artifact(s)
- transcript artifacts
- draft template profile
- verifier identity metadata

**Outputs:**
- `exports/healthcareos/deliverables/draft_note.md`
- `exports/healthcareos/deliverables/verification_checklist.md`
- `exports/healthcareos/attachments/consent_record.json`
- `exports/healthcareos/attachments/citations_map.json`
- `exports/healthcareos/attachments/uncertainty_map.json`

### 5.3 State Management

- Use existing run lifecycle for top-level states.
- Inside each pack module, maintain explicit workflow state structs:
  - `Ingested`
  - `Analyzed`
  - `Reviewed`
  - `Renderable`
  - `ExportReady`
- Transition validation is strict (invalid transition => typed error + audit event).

---

## 6) Implementation Order with Explicit Dependencies

## Stage 0 (Prerequisite Hardening)

1. Freeze baseline on green: `cargo test --workspace`, `pnpm gate:all`.
2. Add placeholders for pack modules and wire `lib.rs` exports.
3. Add pack command DTO shells in `src-tauri/src/main.rs`.

**Dependency:** Must complete before any pack-specific logic.

## Stage 1 (Phase 4 RedlineOS)

1. Implement RedlineOS models and ingestion.
2. Implement deterministic anchor generator (`anchors.rs`).
3. Implement workflow and render outputs.
4. Wire run manager integration.
5. Add RedlineOS eval gates + gate runner scenario.
6. Add tests and fixtures; pass exit gate.

**Dependency:** Stage 0 complete.

## Stage 2 (Phase 5 IncidentOS)

1. Implement untrusted input sanitization path first (`sanitize.rs`).
2. Build timeline workflow from artifact refs.
3. Implement dual-template rendering.
4. Enforce customer redaction gating in export flow.
5. Add IncidentOS eval gates + tests.

**Dependency:** Stage 1 complete and stable.

## Stage 3 (Phase 6 FinanceOS)

1. Implement finance input normalization.
2. Implement deterministic exception detection output.
3. Implement retention + redaction defaults.
4. Add downstream CSV exporter.
5. Add FinanceOS gates + tests.

**Dependency:** Stage 2 complete.

## Stage 4 (Phase 7 HealthcareOS, optional)

1. Implement consent validation and storage.
2. Implement transcript-to-draft with citations.
3. Implement verification checklist and acknowledgement capture.
4. Add strict export block when consent/verification missing.
5. Add HealthcareOS gates + tests.

**Dependency:** explicit product approval + Stage 3 complete.

## Stage 5 (Closure and Promotion for each phase)

1. Update compliance map rows.
2. Produce closure report with commands/results.
3. Run full gate suite and deterministic dual-run proof.
4. Tag phase complete only when all blocker gates pass.

---

## 7) Error Handling Strategy and Edge Cases

### 7.1 Error Taxonomy

Add/extend typed errors for:

- `InputSchemaError`
- `ArtifactMissingError`
- `PolicyViolationError`
- `DeterminismViolationError`
- `CitationViolationError`
- `RedactionViolationError`
- `ConsentMissingError` (Phase 7)
- `WorkflowTransitionError`

All errors must:

- be returned as structured result types,
- create audit events with machine-readable detail payload,
- block export when policy mandates.

### 7.2 Edge-Case Matrix

- Corrupt archive/PDF parse partial failure: mark artifact invalid, continue only if policy allows.
- Duplicate artifact IDs: hard fail (determinism risk).
- Citation refers to redacted segment not present: fail strict mode.
- Redaction required but map missing: block export.
- Cross-platform extraction drift (RedlineOS): gate fail.
- Incident log prompt-injection text: treat as inert evidence text.
- Finance numeric locale drift (`,` vs `.`): normalize and record conversion rule.
- Healthcare consent timestamp after export attempt: block export.

---

## 8) Integration Points and Test Strategy

### 8.1 Integration Points

1. UI -> Tauri command invocation and DTO serialization.
2. Tauri -> Core run API.
3. Core -> EvidenceBundle builder.
4. Core -> Eval runner and validator.
5. Gate runner -> Pack scenarios.

### 8.2 Test Matrix

Per phase, require:

- Unit tests (models, workflow transitions, render determinism).
- Integration tests (`core/tests/*_pack.rs`) for end-to-end bundle generation.
- Gate tests in `pnpm gate:all` for pack-specific blocker gates.
- Determinism tests: two consecutive identical runs hash-equal for deterministic artifacts.
- UI smoke path tests in `src/smoke.test.ts`.

### 8.3 Exit Criteria (each phase)

A phase is complete only when:

1. New pack gates pass (`BLOCKER` all PASS/NOT_APPLICABLE as applicable).
2. Bundle validator passes.
3. Determinism assertions pass for deterministic artifacts.
4. No regression in existing Phase 2/3 gates.
5. Closure doc and compliance map updated.

---

## 9) Missing Dependencies and Logical Risks (Now Explicit)

1. **Golden corpus ownership** (Phase 4) was unstated.  
   - **Decision:** assign corpus stewardship to QA + domain lead; corpus changes require approval.
2. **Template version control** (Phases 5–7) was unstated.  
   - **Decision:** template IDs versioned and emitted in `templates_used.json`.
3. **Data retention policy source** (Phase 6) was unstated.  
   - **Decision:** retention profiles stored as policy artifacts and hashed in bundle.
4. **Clinical workflow governance** (Phase 7) was unstated.  
   - **Decision:** Phase 7 cannot start without formal legal/compliance sign-off artifact.

---

## 10) Assumptions (Explicit)

1. Existing Phase 2/3 baseline remains green and authoritative.
2. `gates_registry_v3` remains active registry version.
3. Existing validator remains the source of truth for required bundle checks.
4. UI can be evolved without introducing direct network egress.
5. PDF deterministic generation path used in Phase 3 can be reused where needed.
6. Team has capacity for one phase at a time; no parallel major-phase implementation.

If any assumption changes, pause implementation and re-baseline this plan.

---

## 11) Final Approval Statement

**Approval: YES, with revisions integrated above.**  
This plan is now execution-ready for delegation. A competent engineer should be able to implement Phases 4–7 without clarification requests if they follow this document and existing lock documents.

### Reviewer Judgment Calls Made

1. Chose strict pack module isolation over shared mega-module design.
2. Chose sequential phase delivery to reduce regression and policy drift risk.
3. Elevated sanitization-first ordering in IncidentOS due to threat model.
4. Treated Phase 7 as compliance-gated and optional by explicit governance control.

